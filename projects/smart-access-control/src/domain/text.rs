//! Validated, human-entered text fields (names, locations).

use super::DomainError;

/// Trims `raw` and checks it is non-empty, at most `max_chars` characters
/// (Unicode scalar values, not bytes) and free of control characters.
pub(crate) fn validate_text(
    field: &'static str,
    raw: &str,
    max_chars: usize,
    too_long: &'static str,
) -> Result<String, DomainError> {
    let invalid = |reason| DomainError::Validation { field, reason };

    let text = raw.trim();
    if text.is_empty() {
        return Err(invalid("must not be empty"));
    }
    if text.chars().count() > max_chars {
        return Err(invalid(too_long));
    }
    if text.chars().any(char::is_control) {
        return Err(invalid("must not contain control characters"));
    }
    Ok(text.to_owned())
}

/// Declares a validated text newtype:
///
/// ```ignore
/// bounded_text!(UserName, field = "name", max = 100);
/// ```
///
/// The macro expands to a struct with a private `String`, so the only way to
/// build one is `parse`, and every value in the program is known to be valid
/// ("parse, don't validate"). `concat!`/`stringify!` build the "too long"
/// message at compile time, so it stays a `&'static str`.
macro_rules! bounded_text {
    ($(#[$meta:meta])* $name:ident, field = $field:literal, max = $max:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            pub const MAX_CHARS: usize = $max;

            pub fn parse(raw: &str) -> Result<Self, $crate::domain::DomainError> {
                $crate::domain::text::validate_text(
                    $field,
                    raw,
                    $max,
                    concat!("must be at most ", stringify!($max), " characters"),
                )
                .map(Self)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

// Makes the macro usable by path (`super::text::bounded_text!`) elsewhere in the crate.
pub(crate) use bounded_text;

#[cfg(test)]
mod tests {
    use super::*;

    bounded_text!(Label, field = "label", max = 5);

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(Label::parse("  abc \n").unwrap().as_str(), "abc");
    }

    #[test]
    fn rejects_empty_and_blank() {
        for raw in ["", "   ", "\t\n"] {
            assert_eq!(
                Label::parse(raw),
                Err(DomainError::Validation {
                    field: "label",
                    reason: "must not be empty"
                })
            );
        }
    }

    #[test]
    fn length_is_counted_in_characters_not_bytes() {
        assert_eq!(Label::MAX_CHARS, 5);
        // 5 characters, 10 bytes.
        assert!(Label::parse("ééééé").is_ok());
        assert_eq!(
            Label::parse("abcdef"),
            Err(DomainError::Validation {
                field: "label",
                reason: "must be at most 5 characters"
            })
        );
    }

    #[test]
    fn rejects_inner_control_characters() {
        assert!(Label::parse("a\u{0}b").is_err());
        assert!(Label::parse("a\nb").is_err());
    }
}
