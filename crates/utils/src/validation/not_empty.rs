use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
    HashSet,
};

use crate::validation::Validator;

// =================================================================================================
// Not Empty
// =================================================================================================

/// Validates that a value is not empty.
pub struct NotEmpty;

impl<T> Validator<T> for NotEmpty
where
    T: IsEmptyValidation,
{
    fn validate(&self, value: &T) -> Option<&str> {
        value.is_empty_validation().then_some("empty")
    }
}

// -------------------------------------------------------------------------------------------------

// Supporting Trait

trait IsEmptyValidation {
    /// .
    fn is_empty_validation(&self) -> bool;
}

impl<T, const N: usize> IsEmptyValidation for [T; N] {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl<T, U> IsEmptyValidation for BTreeMap<T, U> {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmptyValidation for BTreeSet<T> {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl<T, U, S> IsEmptyValidation for HashMap<T, U, S> {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl<T, S> IsEmptyValidation for HashSet<T, S> {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl IsEmptyValidation for String {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmptyValidation for Vec<T> {
    fn is_empty_validation(&self) -> bool {
        self.is_empty()
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::collections::{
        BTreeMap,
        BTreeSet,
        HashMap,
        HashSet,
    };

    use assertables::{
        assert_none,
        assert_some_eq,
    };

    use crate::validation::{
        Validator as _,
        not_empty::NotEmpty,
    };

    // Array

    #[test]
    fn not_empty_array_valid() {
        let validator = NotEmpty;
        let value = [1, 2, 3];

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_array_invalid() {
        let validator = NotEmpty;
        let value: [i32; 0] = [];

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_array_valid_single_element() {
        let validator = NotEmpty;
        let value = [42];

        assert_none!(validator.validate(&value));
    }

    // BTreeMap

    #[test]
    fn not_empty_btreemap_valid() {
        let validator = NotEmpty;
        let value = BTreeMap::from_iter([(1, "one"), (2, "two")]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_btreemap_invalid() {
        let validator = NotEmpty;
        let value: BTreeMap<i32, &str> = BTreeMap::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_btreemap_valid_single_entry() {
        let validator = NotEmpty;
        let value = BTreeMap::from_iter([(42, "answer")]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_btreemap_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value = BTreeMap::from_iter([(1, "one"), (2, "two")]);

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    // BTreeSet

    #[test]
    fn not_empty_btreeset_valid() {
        let validator = NotEmpty;
        let value = BTreeSet::from_iter([1, 2, 3]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_btreeset_invalid() {
        let validator = NotEmpty;
        let value: BTreeSet<i32> = BTreeSet::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_btreeset_valid_single_element() {
        let validator = NotEmpty;
        let value = BTreeSet::from_iter([42]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_btreeset_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value = BTreeSet::from_iter([1, 2, 3]);

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    // HashMap

    #[test]
    fn not_empty_hashmap_valid() {
        let validator = NotEmpty;
        let value: HashMap<i32, &str> = HashMap::from_iter([(1, "one"), (2, "two")]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_hashmap_invalid() {
        let validator = NotEmpty;
        let value: HashMap<i32, &str> = HashMap::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_hashmap_valid_single_entry() {
        let validator = NotEmpty;
        let value: HashMap<i32, &str> = HashMap::from_iter([(42, "answer")]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_hashmap_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value: HashMap<i32, &str> = HashMap::from_iter([(1, "one"), (2, "two")]);

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    // HashSet

    #[test]
    fn not_empty_hashset_valid() {
        let validator = NotEmpty;
        let value: HashSet<i32> = HashSet::from_iter([1, 2, 3]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_hashset_invalid() {
        let validator = NotEmpty;
        let value: HashSet<i32> = HashSet::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_hashset_valid_single_element() {
        let validator = NotEmpty;
        let value: HashSet<i32> = HashSet::from_iter([42]);

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_hashset_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value: HashSet<i32> = HashSet::from_iter([1, 2, 3]);

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    // String

    #[test]
    fn not_empty_string_valid() {
        let validator = NotEmpty;
        let value = String::from("Hello");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_string_invalid() {
        let validator = NotEmpty;
        let value = String::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_string_valid_single_character() {
        let validator = NotEmpty;
        let value = String::from("a");

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_string_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value = String::from("Hello");

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    // Vec

    #[test]
    fn not_empty_vec_valid() {
        let validator = NotEmpty;
        let value = vec![1, 2, 3];

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_vec_invalid() {
        let validator = NotEmpty;
        let value: Vec<i32> = Vec::new();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }

    #[test]
    fn not_empty_vec_valid_single_element() {
        let validator = NotEmpty;
        let value = vec![42];

        assert_none!(validator.validate(&value));
    }

    #[test]
    fn not_empty_vec_invalid_after_clear() {
        let validator = NotEmpty;
        let mut value = vec![1, 2, 3];

        value.clear();

        assert_some_eq!(Some("empty"), validator.validate(&value));
    }
}
