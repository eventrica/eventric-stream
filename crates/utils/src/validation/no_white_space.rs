use crate::validation::Validator;

// =================================================================================================
// White Space
// =================================================================================================

/// Validates that a value does not contain white space characters.
pub struct NoWhiteSpace;

impl<T> Validator<T> for NoWhiteSpace
where
    T: WhiteSpaceValidation,
{
    fn validate(&self, value: &T) -> Option<&str> {
        value.white_space_validation().then_some("whitespace")
    }
}

/// Validates that a value does not contain preceding white space characters.
pub struct NoPrecedingWhiteSpace;

impl<T> Validator<T> for NoPrecedingWhiteSpace
where
    T: PrecedingWhiteSpaceValidation,
{
    fn validate(&self, value: &T) -> Option<&str> {
        value
            .preceding_white_space_validation()
            .then_some("preceding whitespace")
    }
}

/// Validates that a value does not contain trailing white space characters.
pub struct NoTrailingWhiteSpace;

impl<T> Validator<T> for NoTrailingWhiteSpace
where
    T: TrailingWhiteSpaceValidation,
{
    fn validate(&self, value: &T) -> Option<&str> {
        value
            .trailing_white_space_validation()
            .then_some("preceding whitespace")
    }
}

// -------------------------------------------------------------------------------------------------

// Supporting Traits

trait WhiteSpaceValidation {
    fn white_space_validation(&self) -> bool;
}

impl WhiteSpaceValidation for String {
    fn white_space_validation(&self) -> bool {
        self.contains(char::is_whitespace)
    }
}

trait PrecedingWhiteSpaceValidation {
    fn preceding_white_space_validation(&self) -> bool;
}

impl PrecedingWhiteSpaceValidation for String {
    fn preceding_white_space_validation(&self) -> bool {
        self.starts_with(char::is_whitespace)
    }
}

trait TrailingWhiteSpaceValidation {
    fn trailing_white_space_validation(&self) -> bool;
}

impl TrailingWhiteSpaceValidation for String {
    fn trailing_white_space_validation(&self) -> bool {
        self.ends_with(char::is_whitespace)
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use assertables::{
        assert_none,
        assert_some_eq,
    };

    use crate::validation::{
        Validator as _,
        no_white_space::{
            NoPrecedingWhiteSpace,
            NoTrailingWhiteSpace,
            NoWhiteSpace,
        },
    };

    // No White Space

    #[test]
    fn no_white_space_valid() {
        let validator = NoWhiteSpace;
        let value = String::from("HelloWorld");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_white_space_invalid_space() {
        let validator = NoWhiteSpace;
        let value = String::from("Hello World");

        assert_some_eq!(Some("whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_white_space_invalid_tab() {
        let validator = NoWhiteSpace;
        let value = String::from("Hello\tWorld");

        assert_some_eq!(Some("whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_white_space_invalid_newline() {
        let validator = NoWhiteSpace;
        let value = String::from("Hello\nWorld");

        assert_some_eq!(Some("whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_white_space_invalid_preceding() {
        let validator = NoWhiteSpace;
        let value = String::from(" Hello");

        assert_some_eq!(Some("whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_white_space_invalid_trailing() {
        let validator = NoWhiteSpace;
        let value = String::from("Hello ");

        assert_some_eq!(Some("whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_white_space_valid_empty() {
        let validator = NoWhiteSpace;
        let value = String::new();

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_white_space_valid_single_character() {
        let validator = NoWhiteSpace;
        let value = String::from("a");

        assert_none!(validator.validate(&value));
    }

    // No Preceding White Space

    #[test]
    fn no_preceding_white_space_valid() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_invalid_space() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from(" Hello");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_invalid_tab() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("\tHello");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_invalid_newline() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("\nHello");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_valid_trailing() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("Hello ");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_valid_middle() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_valid_empty() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::new();

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_preceding_white_space_valid_single_character() {
        let validator = NoPrecedingWhiteSpace;
        let value = String::from("a");

        assert_none!(validator.validate(&value));
    }

    // No Trailing White Space

    #[test]
    fn no_trailing_white_space_valid() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_invalid_space() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("Hello ");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_invalid_tab() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("Hello\t");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_invalid_newline() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("Hello\n");

        assert_some_eq!(Some("preceding whitespace"), validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_valid_preceding() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from(" Hello");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_valid_middle() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_valid_empty() {
        let validator = NoTrailingWhiteSpace;
        let value = String::new();

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_trailing_white_space_valid_single_character() {
        let validator = NoTrailingWhiteSpace;
        let value = String::from("a");

        assert_none!(validator.validate(&value));
    }
}
