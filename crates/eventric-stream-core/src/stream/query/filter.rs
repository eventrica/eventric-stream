mod algorithm;
mod point;

use std::{
    cmp::Ordering,
    collections::{
        BTreeSet,
        HashMap,
    },
    ops::Range,
};

use crate::{
    event::{
        PersistentEventHash,
        Version,
        identifier::IdentifierHash,
        tag::TagHash,
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

type Filters = Vec<(Range<Version>, Predicate)>;
type Predicate = Option<BTreeSet<TagHash>>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Filter {
    filters: HashMap<IdentifierHash, Filters>,
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
                            .entry(specifier.0)
                            .or_insert_with(|| (Vec::new(), Vec::new()));

                        untagged.push(specifier.1.clone());
                    }
                }

                // Add a version range to the second vector, containing version ranges paired with a
                // set of Tag hashes.
                SelectorHash::SpecifiersAndTags(specifiers, tags) => {
                    for specifier in specifiers {
                        let (_, tagged) = filters
                            .entry(specifier.0)
                            .or_insert_with(|| (Vec::new(), Vec::new()));

                        tagged.push((specifier.1.clone(), tags.clone()));
                    }
                }
            }
        }

        let filters = filters
            .into_iter()
            .map(|(key, (untagged, _tagged))| {
                let untagged = algorithm::normalize_version_ranges(&untagged)
                    .into_iter()
                    .map(|range| (range, None))
                    .collect();

                // let tagged = tagged.iter().into_group_map_by(|(range, tags)| tags);

                (key, untagged)
            })
            .collect();

        Self { filters }
    }
}

impl Matches for Filter {
    fn matches(&self, event: &PersistentEventHash) -> bool {
        match self.filters.get(&event.identifier) {
            Some(ranges) => ranges.matches(event),
            None => false,
        }
    }
}

impl Matches for Filters {
    #[rustfmt::skip]
    fn matches(&self, event: &PersistentEventHash) -> bool {
        for (range, tags) in self {
            match event.version.partial_cmp(range).unwrap() {
                Ordering::Equal => if tags.as_ref().is_none_or(|tags| tags.is_subset(&event.tags)) {
                    return true;
                }
                Ordering::Greater => return false,
                Ordering::Less => {}
            }
        }

        false
    }
}
