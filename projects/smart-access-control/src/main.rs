use std::process::ExitCode;

use smart_access_control::{
    application::CreateAdministrator,
    config::AppConfig,
    domain::Role,
    error_chain,
    infrastructure::telemetry::{self, LogFormat},
};

const USAGE: &str = "usage:
  smart-access-control                                start the API server
  smart-access-control create-admin <username> [admin|viewer]
                                                      create an administrator;
                                                      the password is read from stdin
  smart-access-control healthcheck                    exit 0 if the local server is ready
                                                      (for container health checks)";

#[tokio::main]
async fn main() -> ExitCode {
    // A missing `.env` file is fine: production supplies real env variables.
    let _ = dotenvy::dotenv();
    let (log_format, unknown_format) =
        LogFormat::parse(std::env::var("LOG_FORMAT").ok().as_deref());
    telemetry::init(log_format);
    if unknown_format {
        tracing::warn!("unknown LOG_FORMAT; expected 'pretty' or 'json', using 'pretty'");
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Command::Serve,
        // Before loading the configuration: needs only APP_PORT.
        ["healthcheck"] => return healthcheck(),
        ["create-admin", username] => Command::CreateAdmin(username.to_string(), Role::Admin),
        ["create-admin", username, role] => match role.parse() {
            Ok(role) => Command::CreateAdmin(username.to_string(), role),
            Err(err) => return usage_error(&err.to_string()),
        },
        _ => return usage_error("unknown arguments"),
    };

    let config = match AppConfig::from_env() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!(%err, "invalid configuration");
            return ExitCode::FAILURE;
        }
    };

    match command {
        Command::Serve => match smart_access_control::run(config).await {
            Ok(()) => {
                tracing::info!("shut down cleanly");
                ExitCode::SUCCESS
            }
            Err(err) => {
                tracing::error!(error = %error_chain(&err), "startup failed");
                ExitCode::FAILURE
            }
        },
        Command::CreateAdmin(username, role) => create_admin(config, username, role).await,
    }
}

enum Command {
    Serve,
    CreateAdmin(String, Role),
}

/// Asks the local server's readiness endpoint, with plain HTTP/1.1 over a
/// `std` socket: the runtime image has no shell or curl, and this needs
/// neither an HTTP client crate nor the async runtime.
fn healthcheck() -> ExitCode {
    use std::{
        io::{Read, Write},
        net::{Ipv4Addr, SocketAddr, TcpStream},
        time::Duration,
    };

    let port = std::env::var("APP_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let timeout = Duration::from_secs(3);

    let status_line = (|| -> std::io::Result<String> {
        let mut stream = TcpStream::connect_timeout(&addr, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.write_all(
            b"GET /api/v1/health/ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )?;
        let mut response = Vec::new();
        stream.take(512).read_to_end(&mut response)?;
        let response = String::from_utf8_lossy(&response);
        Ok(response.lines().next().unwrap_or_default().to_owned())
    })();

    match status_line {
        Ok(line) if line.starts_with("HTTP/1.1 200 ") => ExitCode::SUCCESS,
        Ok(line) => {
            eprintln!("not ready: {line}");
            ExitCode::FAILURE
        }
        Err(err) => {
            eprintln!("not ready: {err}");
            ExitCode::FAILURE
        }
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("error: {message}\n\n{USAGE}");
    ExitCode::from(2)
}

async fn create_admin(config: AppConfig, username: String, role: Role) -> ExitCode {
    // Reading stdin blocks, so do it off the async runtime threads.
    let password = tokio::task::spawn_blocking(|| {
        eprint!("Password for the new administrator (min. 12 characters): ");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map(|_| line)
    })
    .await;
    let password = match password {
        Ok(Ok(line)) => line.trim_end_matches(['\r', '\n']).to_owned(),
        _ => {
            eprintln!("error: could not read the password from stdin");
            return ExitCode::FAILURE;
        }
    };

    let cmd = CreateAdministrator {
        username,
        password,
        role,
    };
    match smart_access_control::create_administrator(config, cmd).await {
        Ok(admin) => {
            eprintln!(
                "\ncreated administrator '{}' with role '{}'",
                admin.username(),
                admin.role().as_str()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("\nerror: {}", error_chain(&err));
            ExitCode::FAILURE
        }
    }
}
