//! Environment-based application configuration.
//!
//! Configuration is read once at startup and passed down explicitly, so no
//! other module needs to touch `std::env`.

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use thiserror::Error;

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_ACCESS_TOKEN_TTL_SECONDS: u32 = 15 * 60;
const DEFAULT_REFRESH_TOKEN_TTL_SECONDS: u32 = 7 * 24 * 60 * 60;
pub const DEFAULT_CONTROLLER_TIMEOUT_SECONDS: u32 = 30;
pub const DEFAULT_WS_MAX_CONNECTIONS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    /// Browser origins allowed to call the API (CORS). Empty = same-origin only.
    pub cors_allowed_origins: Vec<String>,
    /// A controller silent for this long is marked offline with its doors.
    pub controller_timeout: chrono::Duration,
    /// Mounts `/simulator/*` and runs simulated controllers in-process.
    pub simulator_enabled: bool,
    /// Upper bound on concurrent live-event WebSocket connections.
    pub ws_max_connections: usize,
    /// Enables `GET /metrics` for scrapers presenting this bearer token.
    /// `None` (the default): the endpoint does not exist.
    pub metrics_token: Option<MetricsToken>,
    /// Reverse proxies whose `X-Forwarded-For` header is believed. Empty
    /// (the default): the TCP peer address is the client address.
    pub trusted_proxies: TrustedProxies,
    /// The built dashboard (`dashboard/dist`), served at `/` when set.
    pub static_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub jwt_secret: JwtSecret,
    pub access_token_ttl: chrono::Duration,
    pub refresh_token_ttl: chrono::Duration,
    /// Mark the refresh-token cookie `Secure` (HTTPS only). Browsers also
    /// accept `Secure` cookies on http://localhost, so this stays `true`
    /// unless the dashboard is served over plain HTTP from another host.
    pub cookie_secure: bool,
}

/// The HMAC key that signs access tokens. Anyone holding it can mint tokens
/// for any administrator, so it is redacted in `Debug` and never logged.
#[derive(Clone, PartialEq, Eq)]
pub struct JwtSecret(String);

impl JwtSecret {
    /// 32 bytes = 256 bits, matching the HS256 output size.
    pub const MIN_BYTES: usize = 32;
    /// The placeholder shipped in `.env.example`.
    const EXAMPLE_PLACEHOLDER: &str = "replace-me";

    pub fn new(secret: impl Into<String>) -> Result<Self, ConfigError> {
        let secret = secret.into();
        if secret.len() < Self::MIN_BYTES {
            return Err(ConfigError::Insecure {
                key: "JWT_SECRET",
                reason: "must be at least 32 bytes (e.g. `openssl rand -hex 32`)",
            });
        }
        // The value from `.env.example` is long enough but public: anyone
        // could forge tokens with it.
        if secret.contains(Self::EXAMPLE_PLACEHOLDER) {
            return Err(ConfigError::Insecure {
                key: "JWT_SECRET",
                reason: "is still the example placeholder; generate one with `openssl rand -hex 32`",
            });
        }
        Ok(Self(secret))
    }

    pub fn expose(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JwtSecret(<redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: DatabaseUrl,
    pub max_connections: u32,
}

/// Bearer token that Prometheus must present to read `GET /metrics`.
/// Redacted in `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub struct MetricsToken(String);

impl MetricsToken {
    pub const MIN_BYTES: usize = 16;

    pub fn new(token: impl Into<String>) -> Result<Self, ConfigError> {
        let token = token.into();
        if token.len() < Self::MIN_BYTES {
            return Err(ConfigError::Insecure {
                key: "METRICS_TOKEN",
                reason: "must be at least 16 bytes (e.g. `openssl rand -hex 16`)",
            });
        }
        Ok(Self(token))
    }

    /// Compares in constant time, so response timing does not reveal how
    /// many leading characters of a guess were right.
    pub fn matches(&self, candidate: &str) -> bool {
        let (a, b) = (self.0.as_bytes(), candidate.as_bytes());
        a.len() == b.len() && a.iter().zip(b).fold(0u8, |diff, (x, y)| diff | (x ^ y)) == 0
    }
}

impl fmt::Debug for MetricsToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricsToken(<redacted>)")
    }
}

/// An IP network in CIDR notation: `10.0.0.0/8`, `2001:db8::/32`. A bare
/// address is a network of one (`/32` or `/128`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpNetwork {
    addr: IpAddr,
    prefix: u8,
}

impl IpNetwork {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (addr, prefix) = match raw.split_once('/') {
            Some((addr, prefix)) => (addr, Some(prefix)),
            None => (raw, None),
        };
        let addr: IpAddr = addr
            .parse()
            .map_err(|_| "not an IP address or CIDR network".to_owned())?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            None => max,
            Some(prefix) => prefix
                .parse::<u8>()
                .ok()
                .filter(|&prefix| prefix <= max)
                .ok_or_else(|| format!("prefix length must be 0-{max}"))?,
        };
        // `10.0.0.1/8` is almost certainly a typo for `10.0.0.1` or `10.0.0.0/8`.
        if truncate(addr, prefix) != addr {
            return Err("address has bits set outside the prefix".to_owned());
        }
        Ok(Self { addr, prefix })
    }

    /// Whether `ip` lies in this network. IPv4-mapped IPv6 addresses
    /// (`::ffff:10.0.0.1`, as dual-stack sockets report IPv4 peers) count as
    /// the IPv4 address they wrap.
    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = ip.to_canonical();
        ip.is_ipv4() == self.addr.is_ipv4() && truncate(ip, self.prefix) == self.addr
    }
}

/// Clears every bit after the first `prefix` bits: `truncate(10.1.2.3, 8)`
/// is `10.0.0.0`. The address is handled as one big integer (`u32` or
/// `u128`); `checked_shl` covers `/0`, where shifting by the full width
/// would overflow.
fn truncate(addr: IpAddr, prefix: u8) -> IpAddr {
    let prefix = u32::from(prefix);
    match addr {
        IpAddr::V4(addr) => {
            let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
            IpAddr::V4((u32::from(addr) & mask).into())
        }
        IpAddr::V6(addr) => {
            let mask = u128::MAX.checked_shl(128 - prefix).unwrap_or(0);
            IpAddr::V6((u128::from(addr) & mask).into())
        }
    }
}

/// The reverse proxies allowed to report the client address.
///
/// Only a proxy we run may say who the client is: `X-Forwarded-For` from
/// anyone else is just a header an attacker can set to dodge the per-address
/// login limit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedProxies(Vec<IpNetwork>);

impl TrustedProxies {
    pub fn new(networks: Vec<IpNetwork>) -> Self {
        Self(networks)
    }

    /// Parses a comma-separated list such as `172.30.83.10,10.0.0.0/8`.
    pub fn parse(raw: &str) -> Result<Self, ConfigError> {
        raw.split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(|entry| {
                IpNetwork::parse(entry).map_err(|reason| ConfigError::Invalid {
                    key: "TRUSTED_PROXIES",
                    value: entry.to_owned(),
                    reason,
                })
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        self.0.iter().any(|network| network.contains(ip))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A database connection string. It usually contains a password, so its
/// `Debug` output is redacted: logging the config can never leak it.
#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseUrl(String);

impl DatabaseUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into())
    }

    /// Returns the raw URL. Only pass this to the database driver.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DatabaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DatabaseUrl(<redacted>)")
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("{key} must be set")]
    Missing { key: &'static str },

    /// A secret failed a safety check. The value is deliberately not included.
    #[error("{key} {reason}")]
    Insecure {
        key: &'static str,
        reason: &'static str,
    },

    #[error("{key} has an invalid value {value:?}: {reason}")]
    Invalid {
        key: &'static str,
        value: String,
        reason: String,
    },
}

impl AppConfig {
    /// Reads configuration from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Builds configuration from any key lookup.
    ///
    /// Taking a closure instead of reading `std::env` directly lets tests pass
    /// a fake environment. Mutating the real environment is `unsafe` in
    /// edition 2024 because other threads may be reading it concurrently.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let url = lookup("DATABASE_URL")
            .filter(|url| !url.trim().is_empty())
            .ok_or(ConfigError::Missing {
                key: "DATABASE_URL",
            })?;
        let jwt_secret = lookup("JWT_SECRET").ok_or(ConfigError::Missing { key: "JWT_SECRET" })?;
        let seconds = |key, default| {
            parse_or(&lookup, key, default).map(|secs: u32| chrono::Duration::seconds(secs.into()))
        };

        Ok(Self {
            host: parse_or(&lookup, "APP_HOST", DEFAULT_HOST)?,
            port: parse_or(&lookup, "APP_PORT", DEFAULT_PORT)?,
            database: DatabaseConfig {
                url: DatabaseUrl::new(url),
                max_connections: parse_or(
                    &lookup,
                    "DATABASE_MAX_CONNECTIONS",
                    DEFAULT_DB_MAX_CONNECTIONS,
                )?,
            },
            cors_allowed_origins: parse_origins(lookup("CORS_ALLOWED_ORIGINS"))?,
            simulator_enabled: parse_or(&lookup, "SIMULATOR_ENABLED", false)?,
            metrics_token: lookup("METRICS_TOKEN")
                .filter(|token| !token.trim().is_empty())
                .map(MetricsToken::new)
                .transpose()?,
            trusted_proxies: TrustedProxies::parse(&lookup("TRUSTED_PROXIES").unwrap_or_default())?,
            static_dir: lookup("STATIC_DIR")
                .filter(|dir| !dir.trim().is_empty())
                .map(PathBuf::from),
            ws_max_connections: parse_or(
                &lookup,
                "WS_MAX_CONNECTIONS",
                DEFAULT_WS_MAX_CONNECTIONS,
            )?,
            controller_timeout: seconds(
                "CONTROLLER_TIMEOUT_SECONDS",
                DEFAULT_CONTROLLER_TIMEOUT_SECONDS,
            )?,
            auth: AuthConfig {
                jwt_secret: JwtSecret::new(jwt_secret)?,
                access_token_ttl: seconds(
                    "ACCESS_TOKEN_TTL_SECONDS",
                    DEFAULT_ACCESS_TOKEN_TTL_SECONDS,
                )?,
                refresh_token_ttl: seconds(
                    "REFRESH_TOKEN_TTL_SECONDS",
                    DEFAULT_REFRESH_TOKEN_TTL_SECONDS,
                )?,
                cookie_secure: parse_or(&lookup, "COOKIE_SECURE", true)?,
            },
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

/// Parses a comma-separated origin list such as
/// `http://localhost:5173,https://admin.example.com`.
///
/// Each entry must be exactly `scheme://host[:port]`, as browsers send it in
/// the `Origin` header. `*` is rejected: any website could then call the API.
fn parse_origins(raw: Option<String>) -> Result<Vec<String>, ConfigError> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    raw.split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(|origin| {
            let invalid = |reason: &str| ConfigError::Invalid {
                key: "CORS_ALLOWED_ORIGINS",
                value: origin.to_owned(),
                reason: reason.to_owned(),
            };
            let host = origin
                .strip_prefix("https://")
                .or_else(|| origin.strip_prefix("http://"))
                .ok_or_else(|| invalid("must start with http:// or https://"))?;
            if host.is_empty() || host.contains(['/', '*', ' ', '?', '#']) {
                return Err(invalid(
                    "must be scheme://host[:port], without path or wildcard",
                ));
            }
            Ok(origin.to_owned())
        })
        .collect()
}

/// Parses `key` if set, otherwise returns `default`.
///
/// Generic over any `T: FromStr`, so the same helper handles `IpAddr`, `u16`
/// and future settings. The `Display` bound lets us keep the parser's error
/// message without depending on its concrete type.
fn parse_or<T>(
    lookup: &impl Fn(&str) -> Option<String>,
    key: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match lookup(key) {
        None => Ok(default),
        Some(value) => value.parse().map_err(|err: T::Err| ConfigError::Invalid {
            key,
            reason: err.to_string(),
            value,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: (&str, &str) = ("DATABASE_URL", "postgres://sac:s3cret@localhost/db");
    const JWT: (&str, &str) = ("JWT_SECRET", "0123456789abcdef0123456789abcdef-jwt");

    fn lookup_from<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn uses_defaults_when_unset() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        assert_eq!(config.host, DEFAULT_HOST);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.database.max_connections, DEFAULT_DB_MAX_CONNECTIONS);
        assert_eq!(config.database.url.expose(), URL.1);
    }

    #[test]
    fn reads_values_from_environment() {
        let config = AppConfig::from_lookup(lookup_from(&[
            URL,
            JWT,
            ("APP_HOST", "0.0.0.0"),
            ("APP_PORT", "9000"),
            ("DATABASE_MAX_CONNECTIONS", "3"),
        ]))
        .unwrap();
        assert_eq!(config.socket_addr(), "0.0.0.0:9000".parse().unwrap());
        assert_eq!(config.database.max_connections, 3);
    }

    #[test]
    fn database_url_is_required() {
        for pairs in [&[JWT][..], &[("DATABASE_URL", "  "), JWT][..]] {
            assert_eq!(
                AppConfig::from_lookup(lookup_from(pairs)),
                Err(ConfigError::Missing {
                    key: "DATABASE_URL"
                })
            );
        }
    }

    #[test]
    fn debug_output_never_contains_the_database_password() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        let debug = format!("{config:?}");
        assert!(!debug.contains("s3cret"), "{debug}");
        assert!(!debug.contains(JWT.1), "{debug}");
        assert!(debug.contains("DatabaseUrl(<redacted>)"));
        assert!(debug.contains("JwtSecret(<redacted>)"));
    }

    #[test]
    fn jwt_secret_is_required_and_must_be_long() {
        assert_eq!(
            AppConfig::from_lookup(lookup_from(&[URL])),
            Err(ConfigError::Missing { key: "JWT_SECRET" })
        );
        let err =
            AppConfig::from_lookup(lookup_from(&[URL, ("JWT_SECRET", "too-short")])).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Insecure {
                key: "JWT_SECRET",
                ..
            }
        ));
        // The rejected value never appears in the message.
        assert!(!err.to_string().contains("too-short"));
    }

    #[test]
    fn example_placeholder_secret_is_rejected() {
        // Read the real file so this test breaks if the example changes.
        let example = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.example"))
            .expect(".env.example exists");
        let placeholder = example
            .lines()
            .find_map(|line| line.strip_prefix("JWT_SECRET="))
            .expect(".env.example sets JWT_SECRET");

        let err = JwtSecret::new(placeholder).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Insecure {
                key: "JWT_SECRET",
                ..
            }
        ));
    }

    #[test]
    fn cors_origins() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        assert!(config.cors_allowed_origins.is_empty());

        let config = AppConfig::from_lookup(lookup_from(&[
            URL,
            JWT,
            (
                "CORS_ALLOWED_ORIGINS",
                " http://localhost:5173 , https://admin.example.com:8443,",
            ),
        ]))
        .unwrap();
        assert_eq!(
            config.cors_allowed_origins,
            ["http://localhost:5173", "https://admin.example.com:8443"]
        );

        for bad in [
            "*",
            "localhost:5173",
            "http://",
            "https://a.com/",
            "http://*.a.com",
        ] {
            let result =
                AppConfig::from_lookup(lookup_from(&[URL, JWT, ("CORS_ALLOWED_ORIGINS", bad)]));
            assert!(
                matches!(
                    result,
                    Err(ConfigError::Invalid {
                        key: "CORS_ALLOWED_ORIGINS",
                        ..
                    })
                ),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn metrics_token_is_optional_long_and_redacted() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        assert_eq!(config.metrics_token, None);

        let short = AppConfig::from_lookup(lookup_from(&[URL, JWT, ("METRICS_TOKEN", "short")]));
        assert!(matches!(
            short,
            Err(ConfigError::Insecure {
                key: "METRICS_TOKEN",
                ..
            })
        ));

        let token = MetricsToken::new("scrape-me-please-123").unwrap();
        assert!(token.matches("scrape-me-please-123"));
        assert!(!token.matches("scrape-me-please-124"));
        assert!(!token.matches("scrape"));
        assert_eq!(format!("{token:?}"), "MetricsToken(<redacted>)");
    }

    #[test]
    fn ip_networks() {
        let net = |raw| IpNetwork::parse(raw).unwrap();
        let ip = |raw: &str| raw.parse::<IpAddr>().unwrap();

        assert!(net("10.0.0.0/8").contains(ip("10.255.1.2")));
        assert!(!net("10.0.0.0/8").contains(ip("11.0.0.0")));
        assert!(net("172.30.83.10").contains(ip("172.30.83.10")));
        assert!(!net("172.30.83.10").contains(ip("172.30.83.11")));
        assert!(net("0.0.0.0/0").contains(ip("203.0.113.9")));
        // The full-length prefixes are valid and mean a single address.
        assert!(net("172.30.83.10/32").contains(ip("172.30.83.10")));
        assert!(!net("172.30.83.10/32").contains(ip("172.30.83.11")));
        assert!(net("::1/128").contains(ip("::1")));
        assert!(net("172.16.0.0/12").contains(ip("172.31.255.255")));
        assert!(!net("172.16.0.0/12").contains(ip("172.32.0.0")));
        // IPv4 peers on a dual-stack socket.
        assert!(net("10.0.0.0/8").contains(ip("::ffff:10.1.2.3")));
        assert!(net("2001:db8::/32").contains(ip("2001:db8:ffff::1")));
        assert!(!net("2001:db8::/32").contains(ip("2001:db9::1")));
        assert!(!net("::/0").contains(ip("10.0.0.1")), "families never mix");

        for bad in [
            "",
            "10.0.0.0/33",
            "::/129",
            "10.0.0.0/",
            "10.0.0.1/8",
            "host/24",
            "10.0.0.0/-1",
        ] {
            assert!(IpNetwork::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn trusted_proxies_and_static_dir() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        assert!(config.trusted_proxies.is_empty());
        assert_eq!(config.static_dir, None);

        let config = AppConfig::from_lookup(lookup_from(&[
            URL,
            JWT,
            ("TRUSTED_PROXIES", " 172.30.83.10 , 10.0.0.0/8,"),
            ("STATIC_DIR", "/app/dashboard"),
        ]))
        .unwrap();
        assert!(config.trusted_proxies.contains("10.9.9.9".parse().unwrap()));
        assert!(
            config
                .trusted_proxies
                .contains("172.30.83.10".parse().unwrap())
        );
        assert!(
            !config
                .trusted_proxies
                .contains("172.30.83.11".parse().unwrap())
        );
        assert_eq!(config.static_dir, Some(PathBuf::from("/app/dashboard")));

        let bad = AppConfig::from_lookup(lookup_from(&[URL, JWT, ("TRUSTED_PROXIES", "proxy")]));
        assert!(matches!(
            bad,
            Err(ConfigError::Invalid {
                key: "TRUSTED_PROXIES",
                ..
            })
        ));
    }

    #[test]
    fn token_lifetimes() {
        let config = AppConfig::from_lookup(lookup_from(&[URL, JWT])).unwrap();
        assert_eq!(config.auth.access_token_ttl, chrono::Duration::minutes(15));
        assert_eq!(config.auth.refresh_token_ttl, chrono::Duration::days(7));
        assert_eq!(config.controller_timeout, chrono::Duration::seconds(30));
        assert!(
            !config.simulator_enabled,
            "simulator is off unless asked for"
        );
        assert!(config.auth.cookie_secure, "cookies are Secure by default");

        let config =
            AppConfig::from_lookup(lookup_from(&[URL, JWT, ("ACCESS_TOKEN_TTL_SECONDS", "60")]))
                .unwrap();
        assert_eq!(config.auth.access_token_ttl, chrono::Duration::seconds(60));
    }

    #[test]
    fn rejects_invalid_port() {
        let err =
            AppConfig::from_lookup(lookup_from(&[URL, JWT, ("APP_PORT", "70000")])).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                key: "APP_PORT",
                ..
            }
        ));
    }

    #[test]
    fn rejects_invalid_host() {
        let err = AppConfig::from_lookup(lookup_from(&[URL, JWT, ("APP_HOST", "not-an-ip")]))
            .unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                key: "APP_HOST",
                ..
            }
        ));
    }
}
