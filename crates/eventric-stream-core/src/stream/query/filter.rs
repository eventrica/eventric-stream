mod algorithm;
mod point;

use std::collections::HashMap;

use any_range::AnyRange;
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
trait Matches {
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

                        if let Some(range) = &specifier.range {
                            filter.ranges.push(range.clone());
                        }
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
    ranges: Vec<AnyRange<Version>>,
}

impl Matches for IdentifierLevelFilter {
    fn matches(&self, event: &PersistentEventHash) -> bool {
        if self
            .ranges
            .iter()
            .any(|range| range.contains(&event.version))
        {
            return true;
        }

        false
    }
}
