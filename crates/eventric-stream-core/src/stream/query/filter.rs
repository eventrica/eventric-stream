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
                        filters
                            .entry(specifier.0)
                            .or_insert_with(|| (Vec::new(), Vec::new()))
                            .0
                            .push(specifier.1.clone());
                    }
                }

                // Add a version range to the second vector, containing version ranges paired with a
                // set of Tag hashes.
                SelectorHash::SpecifiersAndTags(specifiers, tags) => {
                    for specifier in specifiers {
                        filters
                            .entry(specifier.0)
                            .or_insert_with(|| (Vec::new(), Vec::new()))
                            .1
                            .push((specifier.1.clone(), tags.clone()));
                    }
                }
            }
        }

        let filters = filters
            .into_iter()
            .map(|(key, (untagged, tagged))| {
                let mut filters = Vec::new();

                filters.append(
                    &mut algorithm::normalize_version_ranges(&untagged)
                        .into_iter()
                        .map(|range| (range, None))
                        .collect(),
                );

                let mut tagged_map = HashMap::new();

                for (range, tags) in tagged {
                    tagged_map.entry(tags).or_insert_with(Vec::new).push(range);
                }

                filters.append(
                    &mut tagged_map
                        .into_iter()
                        .flat_map(|(tags, ranges)| {
                            algorithm::normalize_version_ranges(&ranges)
                                .into_iter()
                                .map(move |range| (range, Some(tags.clone())))
                                .collect::<Vec<_>>()
                        })
                        .collect(),
                );

                (key, filters)
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
