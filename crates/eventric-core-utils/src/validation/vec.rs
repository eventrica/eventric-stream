use crate::validation::Validator;

pub struct IsEmpty;

impl<T> Validator<Vec<T>> for IsEmpty {
    fn validate(&self, value: &Vec<T>) -> Option<&str> {
        value.is_empty().then_some("empty")
    }
}
