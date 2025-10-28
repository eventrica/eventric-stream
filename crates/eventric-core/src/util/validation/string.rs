use crate::util::validation::Validator;

// =================================================================================================
// String
// =================================================================================================

pub struct ControlCharacters;

impl Validator<String> for ControlCharacters {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .chars()
            .any(char::is_control)
            .then_some("control characters")
    }
}

pub struct IsEmpty;

impl Validator<String> for IsEmpty {
    fn validate(&self, value: &String) -> Option<&str> {
        value.is_empty().then_some("empty")
    }
}

pub struct PrecedingWhitespace;

impl Validator<String> for PrecedingWhitespace {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .starts_with(char::is_whitespace)
            .then_some("preceding whitespace")
    }
}

pub struct TrailingWhitespace;

impl Validator<String> for TrailingWhitespace {
    fn validate(&self, value: &String) -> Option<&str> {
        value
            .ends_with(char::is_whitespace)
            .then_some("trailing whitespace")
    }
}
