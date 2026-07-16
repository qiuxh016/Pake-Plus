use regex::Regex;
use std::sync::LazyLock;

use super::source::comparable_application_name;

static CARD_CANDIDATE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[0-9][ -]?){12,18}[0-9]\b").expect("valid card candidate regex")
});

#[derive(Clone, Debug)]
pub struct FilterConfig {
    pub ignore_short_text: bool,
    pub min_length: usize,
    pub max_length: usize,
    pub ignore_password_like: bool,
    pub ignore_credit_card_like: bool,
    pub ignored_apps: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            ignore_short_text: true,
            min_length: 2,
            max_length: 10_000,
            ignore_password_like: true,
            ignore_credit_card_like: true,
            ignored_apps: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterReason {
    TooShort,
    TooLong,
    PasswordLike,
    CreditCardLike,
    IgnoredApplication,
}

pub fn should_record(
    text: &str,
    source_app: Option<&str>,
    config: &FilterConfig,
) -> Result<(), FilterReason> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(FilterReason::TooShort);
    }

    if config.ignore_short_text && trimmed.chars().count() < config.min_length {
        return Err(FilterReason::TooShort);
    }

    if trimmed.chars().count() > config.max_length {
        return Err(FilterReason::TooLong);
    }

    if source_app.is_some_and(|source| {
        let source = comparable_application_name(source);
        config
            .ignored_apps
            .iter()
            .any(|ignored| comparable_application_name(ignored) == source)
    }) {
        return Err(FilterReason::IgnoredApplication);
    }

    if config.ignore_credit_card_like && contains_valid_card_number(trimmed) {
        return Err(FilterReason::CreditCardLike);
    }

    if config.ignore_password_like && is_password_like(trimmed) {
        return Err(FilterReason::PasswordLike);
    }

    Ok(())
}

fn is_password_like(value: &str) -> bool {
    let length = value.chars().count();
    if !(6..=30).contains(&length) || value.chars().any(char::is_whitespace) {
        return false;
    }

    let has_letter = value
        .chars()
        .any(|character| character.is_ascii_alphabetic());
    let has_digit = value.chars().any(|character| character.is_ascii_digit());
    has_letter && has_digit
}

fn contains_valid_card_number(value: &str) -> bool {
    CARD_CANDIDATE_PATTERN.find_iter(value).any(|candidate| {
        let digits: Vec<u32> = candidate
            .as_str()
            .bytes()
            .filter(u8::is_ascii_digit)
            .map(|digit| u32::from(digit - b'0'))
            .collect();
        luhn_is_valid(&digits)
    })
}

fn luhn_is_valid(digits: &[u32]) -> bool {
    if !(13..=19).contains(&digits.len()) {
        return false;
    }

    let parity = digits.len() % 2;
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(index, digit)| {
            if index % 2 == parity {
                let doubled = digit * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                *digit
            }
        })
        .sum();
    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_short_text_when_enabled() {
        let config = FilterConfig::default();
        assert_eq!(
            should_record("a", None, &config),
            Err(FilterReason::TooShort)
        );
    }

    #[test]
    fn records_normal_text() {
        let config = FilterConfig::default();
        assert!(should_record("hello world", None, &config).is_ok());
    }

    #[test]
    fn skips_password_like_text() {
        let config = FilterConfig::default();
        assert_eq!(
            should_record("abc12345", None, &config),
            Err(FilterReason::PasswordLike)
        );
    }

    #[test]
    fn skips_credit_card_like_text() {
        let config = FilterConfig::default();
        assert_eq!(
            should_record("4111 1111 1111 1111", None, &config),
            Err(FilterReason::CreditCardLike)
        );
    }

    #[test]
    fn enforces_length_boundaries() {
        let config = FilterConfig::default();
        assert_eq!(
            should_record("界", None, &config),
            Err(FilterReason::TooShort)
        );
        assert!(should_record("界面", None, &config).is_ok());
        assert!(should_record(&"x".repeat(10_000), None, &config).is_ok());
        assert_eq!(
            should_record(&"x".repeat(10_001), None, &config),
            Err(FilterReason::TooLong)
        );
    }

    #[test]
    fn password_filter_uses_exact_alphanumeric_range() {
        let config = FilterConfig::default();
        assert!(should_record("abcde", None, &config).is_ok());
        assert!(should_record("ordinaryword", None, &config).is_ok());
        assert_eq!(
            should_record("abc123", None, &config),
            Err(FilterReason::PasswordLike)
        );
        assert_eq!(
            should_record("P@ssw0rd", None, &config),
            Err(FilterReason::PasswordLike)
        );
        assert_eq!(
            should_record("abc-123", None, &config),
            Err(FilterReason::PasswordLike)
        );
        assert!(should_record("1234567890", None, &config).is_ok());
        assert!(should_record(&format!("a{}", "1".repeat(30)), None, &config).is_ok());
    }

    #[test]
    fn card_filter_accepts_valid_thirteen_to_nineteen_digit_numbers() {
        let config = FilterConfig::default();
        for value in [
            "4222222222222",
            "378282246310005",
            "4111111111111111",
            "4111-1111-1111-1111",
            "card 4111 1111 1111 1111",
            "4000000000000000006",
        ] {
            assert_eq!(
                should_record(value, None, &config),
                Err(FilterReason::CreditCardLike)
            );
        }
    }

    #[test]
    fn card_filter_rejects_invalid_luhn_numbers_and_out_of_range_lengths() {
        let config = FilterConfig::default();
        for value in [
            "4111111111111112",
            "4111 1111 1111 111",
            "40000000000000000006",
        ] {
            assert!(
                should_record(value, None, &config).is_ok(),
                "unexpected card match for {value}"
            );
        }
    }

    #[test]
    fn content_type_filters_can_be_disabled() {
        let config = FilterConfig {
            ignore_password_like: false,
            ignore_credit_card_like: false,
            ..FilterConfig::default()
        };
        assert!(should_record("abc12345", None, &config).is_ok());
        assert!(should_record("4111 1111 1111 1111", None, &config).is_ok());
    }

    #[test]
    fn skips_configured_source_app_case_insensitively() {
        let config = FilterConfig {
            ignored_apps: vec!["Notepad".to_string(), "Visual Studio Code".to_string()],
            ..FilterConfig::default()
        };
        assert_eq!(
            should_record("normal text", Some("NOTEPAD.EXE"), &config),
            Err(FilterReason::IgnoredApplication)
        );
        assert!(should_record("normal text", Some("firefox.exe"), &config).is_ok());
    }
}
