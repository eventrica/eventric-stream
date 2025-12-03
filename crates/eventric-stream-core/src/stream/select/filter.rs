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
        EventHash,
        Version,
        identifier::IdentifierHash,
        tag::TagHash,
    },
    stream::select::{
        SelectionHash,
        SelectorHash,
    },
};

// =================================================================================================
// Filter
// =================================================================================================

// Matches

pub trait Matches {
    fn matches(&self, event: &EventHash) -> bool;
}

// -------------------------------------------------------------------------------------------------

// Event Level Filter

type Filters = Vec<(Range<Version>, Predicate)>;
type Predicate = Option<BTreeSet<TagHash>>;

#[derive(Debug)]
pub struct Filter {
    filters: HashMap<IdentifierHash, Filters>,
}

impl Filter {
    pub fn new(selection: &SelectionHash) -> Self {
        let mut filters = HashMap::new();

        for selector in &selection.0 {
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
    fn matches(&self, event: &EventHash) -> bool {
        match self.filters.get(&event.identifier) {
            Some(ranges) => ranges.matches(event),
            None => false,
        }
    }
}

impl Matches for Filters {
    #[rustfmt::skip]
    fn matches(&self, event: &EventHash) -> bool {
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

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::{
        event::{
            Data,
            EventHash,
            Position,
            Timestamp,
            Version,
            identifier::Identifier,
            specifier::{
                Specifier,
                SpecifierHash,
            },
            tag::{
                Tag,
                TagHash,
            },
        },
        stream::select::{
            SelectionHash,
            SelectorHash,
            filter::{
                Filter,
                Matches,
            },
        },
    };

    // Helper functions

    fn make_event(identifier: &str, version: u8, tags: Vec<&str>) -> EventHash {
        let identifier = (Identifier::new_unvalidated(identifier)).into();
        let tags = tags
            .into_iter()
            .map(|tag| Tag::new_unvalidated(tag).into())
            .collect();

        EventHash::new(
            Data::new_unvalidated(vec![0]),
            identifier,
            Position::new(0),
            tags,
            Timestamp::new(0),
            Version::new(version),
        )
    }

    fn make_specifier_hash(identifier: &str, range: std::ops::Range<Version>) -> SpecifierHash {
        let id = Identifier::new_unvalidated(identifier);
        let spec = Specifier::new(id).range(range);

        spec.into()
    }

    fn make_tag_hash(tag: &str) -> TagHash {
        Tag::new_unvalidated(tag).into()
    }

    // Filter::new with Specifiers only

    #[test]
    fn new_with_single_specifier() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);

        let filter = Filter::new(&query);

        assert!(filter.filters.len() == 1);
    }

    #[test]
    fn new_with_multiple_specifiers_same_identifier() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("Event", Version::new(10)..Version::new(15));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);

        let filter = Filter::new(&query);

        assert!(filter.filters.len() == 1);
    }

    #[test]
    fn new_with_multiple_specifiers_different_identifiers() {
        let spec1 = make_specifier_hash("EventA", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("EventB", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);

        let filter = Filter::new(&query);

        assert!(filter.filters.len() == 2);
    }

    // Filter::new with SpecifiersAndTags

    #[test]
    fn new_with_specifiers_and_tags() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);

        let filter = Filter::new(&query);

        assert!(filter.filters.len() == 1);
    }

    #[test]
    fn new_with_multiple_tags() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag1 = make_tag_hash("user:123");
        let tag2 = make_tag_hash("org:456");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag1, tag2].into_iter().collect(),
        )]);

        let filter = Filter::new(&query);

        assert!(filter.filters.len() == 1);
    }

    // Filter::matches - basic matching

    #[test]
    fn matches_event_with_matching_identifier_and_version() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_with_wrong_identifier() {
        let spec = make_specifier_hash("EventA", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("EventB", 3, vec![]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_with_version_before_range() {
        let spec = make_specifier_hash("Event", Version::new(5)..Version::new(10));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_with_version_after_range() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 10, vec![]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn matches_event_at_range_start() {
        let spec = make_specifier_hash("Event", Version::new(5)..Version::new(10));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 5, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_at_range_end() {
        let spec = make_specifier_hash("Event", Version::new(5)..Version::new(10));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 10, vec![]);

        assert!(!filter.matches(&event));
    }

    // Filter::matches - tag matching

    #[test]
    fn matches_event_with_exact_tags() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec!["user:123"]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn matches_event_with_superset_of_tags() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec!["user:123", "org:456"]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_missing_required_tag() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec!["org:456"]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_with_no_tags_when_tags_required() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn matches_event_with_all_required_tags() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag1 = make_tag_hash("user:123");
        let tag2 = make_tag_hash("org:456");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag1, tag2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec!["user:123", "org:456"]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_missing_one_required_tag() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag1 = make_tag_hash("user:123");
        let tag2 = make_tag_hash("org:456");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag1, tag2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec!["user:123"]);

        assert!(!filter.matches(&event));
    }

    // Filter::matches - multiple version ranges

    #[test]
    fn matches_event_in_first_of_multiple_ranges() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("Event", Version::new(10)..Version::new(15));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn matches_event_in_merged_overlapping_ranges() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(10));
        let spec2 = make_specifier_hash("Event", Version::new(5)..Version::new(15));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 12, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_event_between_ranges() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("Event", Version::new(10)..Version::new(15));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 7, vec![]);

        assert!(!filter.matches(&event));
    }

    // Filter::matches - overlapping ranges (should be normalized)

    #[test]
    fn matches_event_in_overlapping_ranges() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(10));
        let spec2 = make_specifier_hash("Event", Version::new(5)..Version::new(15));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec1, spec2].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 7, vec![]);

        assert!(filter.matches(&event));
    }

    // Filter::matches - multiple selectors

    #[test]
    fn matches_event_with_first_selector() {
        let spec1 = make_specifier_hash("EventA", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("EventB", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![
            SelectorHash::Specifiers(vec![spec1].into_iter().collect()),
            SelectorHash::Specifiers(vec![spec2].into_iter().collect()),
        ]);
        let filter = Filter::new(&query);

        let event = make_event("EventA", 3, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn matches_event_with_second_selector() {
        let spec1 = make_specifier_hash("EventA", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("EventB", Version::new(1)..Version::new(5));
        let query = SelectionHash::new(vec![
            SelectorHash::Specifiers(vec![spec1].into_iter().collect()),
            SelectorHash::Specifiers(vec![spec2].into_iter().collect()),
        ]);
        let filter = Filter::new(&query);

        let event = make_event("EventB", 3, vec![]);

        assert!(filter.matches(&event));
    }

    // Filter::matches - edge cases

    #[test]
    fn matches_event_with_version_min() {
        let spec = make_specifier_hash("Event", Version::MIN..Version::new(10));
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 0, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn matches_event_with_version_near_max() {
        let spec = make_specifier_hash("Event", Version::new(250)..Version::MAX);
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 254, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn matches_event_with_full_version_range() {
        let spec = make_specifier_hash("Event", Version::MIN..Version::MAX);
        let query = SelectionHash::new(vec![SelectorHash::Specifiers(
            vec![spec].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 100, vec![]);

        assert!(filter.matches(&event));
    }

    // Filter::matches - mixed selectors (with and without tags)

    #[test]
    fn matches_untagged_event_with_specifier_only_selector() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("Event", Version::new(10)..Version::new(15));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![
            SelectorHash::Specifiers(vec![spec1].into_iter().collect()),
            SelectorHash::SpecifiersAndTags(
                vec![spec2].into_iter().collect(),
                vec![tag].into_iter().collect(),
            ),
        ]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(filter.matches(&event));
    }

    #[test]
    fn does_not_match_untagged_event_with_tag_selector_only() {
        let spec = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![SelectorHash::SpecifiersAndTags(
            vec![spec].into_iter().collect(),
            vec![tag].into_iter().collect(),
        )]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 3, vec![]);

        assert!(!filter.matches(&event));
    }

    #[test]
    fn matches_tagged_event_with_tag_selector() {
        let spec1 = make_specifier_hash("Event", Version::new(1)..Version::new(5));
        let spec2 = make_specifier_hash("Event", Version::new(3)..Version::new(15));
        let tag = make_tag_hash("user:123");
        let query = SelectionHash::new(vec![
            SelectorHash::Specifiers(vec![spec1].into_iter().collect()),
            SelectorHash::SpecifiersAndTags(
                vec![spec2].into_iter().collect(),
                vec![tag].into_iter().collect(),
            ),
        ]);
        let filter = Filter::new(&query);

        let event = make_event("Event", 4, vec!["user:123"]);

        assert!(filter.matches(&event));
    }
}
