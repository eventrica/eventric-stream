mod algorithm;
mod point;

use std::{
    cmp::Ordering,
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
                // Add a plain version range to the first vector, containing ranges with no tag
                // specifier
                SelectorHash::Specifiers(specifiers) => {
                    for specifier in specifiers {
                        let (untagged, _) = filters
                            .entry(specifier.0.hash_val())
                            .or_insert_with(|| (Vec::new(), Vec::new()));

                        untagged.push(specifier.1.clone());
                    }
                }

                // Add a version range to the second vector, containing version ranges paired with a
                // set of Tag hashes.
                SelectorHash::SpecifiersAndTags(specifiers, tags) => {
                    for specifier in specifiers {
                        let (_, tagged) = filters
                            .entry(specifier.0.hash_val())
                            .or_insert_with(|| (Vec::new(), Vec::new()));

                        tagged.push((specifier.1.clone(), tags.clone()));
                    }
                }
            }
        }

        let filters = filters
            .into_iter()
            .map(|(key, (untagged, _))| {
                let untagged = algorithm::normalize_version_ranges(&untagged);
                let filter = IdentifierLevelFilter::new(untagged);

                (key, filter)
            })
            .collect();

        Self { filters }
    }
}

impl Matches for Filter {
    fn matches(&self, event: &PersistentEventHash) -> bool {
        match self.filters.get(&event.identifier.hash_val()) {
            Some(filter) => filter.matches(event),
            None => false,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Identifier Level Filter

#[allow(dead_code)]
#[derive(new, Debug)]
#[new(const_fn, vis())]
struct IdentifierLevelFilter {
    ranges: Vec<Range<Version>>,
}

impl Matches for IdentifierLevelFilter {
    #[rustfmt::skip]
    fn matches(&self, event: &PersistentEventHash) -> bool {
        if self.ranges.is_empty() {
            return true;
        }

        for range in &self.ranges {
            match event.version.partial_cmp(range).unwrap() {
                Ordering::Equal => return true,
                Ordering::Greater => break,
                Ordering::Less => {}
            }
        }

        false
    }
}
