use crate::validation::Validator;

// =================================================================================================
// Control Characters
// =================================================================================================

/// Validates that a value does not contain white space characters.
pub struct NoControlCharacters;

impl<T> Validator<T> for NoControlCharacters
where
    T: ControlCharactersValidation,
{
    fn validate(&self, value: &T) -> Option<&str> {
        value
            .control_characters_validation()
            .then_some("control characters")
    }
}

// -------------------------------------------------------------------------------------------------

// Supporting Trait

trait ControlCharactersValidation {
    fn control_characters_validation(&self) -> bool;
}

impl ControlCharactersValidation for String {
    fn control_characters_validation(&self) -> bool {
        self.contains(char::is_control)
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
        no_control_characters::NoControlCharacters,
    };

    // No Control Characters

    #[test]
    fn no_control_characters_valid() {
        let validator = NoControlCharacters;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_newline() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\nWorld");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_tab() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\tWorld");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_carriage_return() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\rWorld");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_null() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\0World");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_bell() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\x07World");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_invalid_escape() {
        let validator = NoControlCharacters;
        let value = String::from("Hello\x1bWorld");

        assert_some_eq!(Some("control characters"), validator.validate(&value));
    }

    #[test]
    fn no_control_characters_valid_with_spaces() {
        let validator = NoControlCharacters;
        let value = String::from("Hello World");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_control_characters_valid_empty() {
        let validator = NoControlCharacters;
        let value = String::new();

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_control_characters_valid_single_character() {
        let validator = NoControlCharacters;
        let value = String::from("a");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_control_characters_valid_with_unicode() {
        let validator = NoControlCharacters;
        let value = String::from("Hello 世界");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn no_control_characters_valid_with_punctuation() {
        let validator = NoControlCharacters;
        let value = String::from("Hello, World!");

        assert_none!(validator.validate(&value));
    }
}
