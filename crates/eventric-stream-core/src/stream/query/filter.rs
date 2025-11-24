mod algorithm;
mod point;

use std::{
    collections::HashMap,
    ops::Range,
};

use fancy_constructor::new;

use crate::{
    event::{
        PersistentEventHash,
        Version,
    },
    stream::query::{
        QueryHash,
        SelectorHash,
    },
};

// =================================================================================================
// Filter
// =================================================================================================

// Matches

#[allow(dead_code)]
pub trait Matches {
    fn matches(&self, event: &PersistentEventHash) -> bool;
}

// -------------------------------------------------------------------------------------------------

// Event Level Filter

#[allow(dead_code)]
#[derive(Debug)]
pub struct Filter {
    filters: HashMap<u64, IdentifierLevelFilter>,
}

impl Filter {
    #[allow(dead_code)]
    pub fn new(query: &QueryHash) -> Self {
        let mut filters = HashMap::new();

        for selector in &query.0 {
            match selector {
                SelectorHash::Specifiers(specifiers) => {
                    for specifier in specifiers {
                        let filter = filters
                            .entry(specifier.identifier.hash())
                            .or_insert_with(IdentifierLevelFilter::new);

                        filter.ranges.push(specifier.range.clone());
                    }
                }
                SelectorHash::SpecifiersAndTags(_specifiers, _tags) => {}
            }
        }

        for filter in filters.values_mut() {
            filter.ranges = algorithm::normalize_version_ranges(&filter.ranges);
        }

        Self { filters }
    }
}

impl Matches for Filter {
    fn matches(&self, event: &PersistentEventHash) -> bool {
        match self.filters.get(&event.identifier.hash()) {
            Some(filter) => filter.matches(event),
            None => false,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier Level Filter

#[allow(dead_code)]
#[derive(new, Debug)]
struct IdentifierLevelFilter {
    #[new(default)]
    ranges: Vec<Range<Version>>,
}

impl Matches for IdentifierLevelFilter {
    #[rustfmt::skip]
    fn matches(&self, event: &PersistentEventHash) -> bool {
        if self.ranges.is_empty() {
            return true;
        }

        for range in &self.ranges {
            match event.version.relative(range) {
                Relative::Equal => return true,
                Relative::GreaterThan => break,
                Relative::LessThan => {}
            }
        }

        false
    }
}

trait RangeRelative {
    fn relative(&self, range: &Range<Self>) -> Relative
    where
        Self: Sized;
}

impl<T> RangeRelative for T
where
    T: PartialOrd,
{
    fn relative(&self, range: &Range<Self>) -> Relative
    where
        Self: Sized,
    {
        if &range.start > self {
            return Relative::LessThan;
        }

        if &range.end <= self {
            return Relative::GreaterThan;
        }

        Relative::Equal
    }
}

pub enum Relative {
    LessThan,
    Equal,
    GreaterThan,
}
