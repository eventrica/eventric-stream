use std::collections::BTreeSet;

use eventric_stream::{
    event::{
        Data,
        Event,
        Facets,
        Name,
        Tag,
        Type,
        Version,
    },
    stream::{
        Stream,
        concurrent::owner::Owner,
        operate::{
            Condition,
            Selection,
            append::Append as _,
            select::{
                Select as _,
                Selector,
                TypeSelector,
            },
        },
    },
    utils::temp_path,
};

fn event(identifier: &str, data: &str, tags: &[&str], version: u8) -> Event<(), String> {
    let ty = Type::new(Name::new(identifier).unwrap(), Version::new(version));
    let tags = tags
        .iter()
        .map(|tag| Tag::new(*tag).unwrap())
        .collect::<BTreeSet<_>>();

    Event::new(Data::new(data).unwrap(), Facets::new(ty, tags), ())
}

pub fn main() {
    let owner = Owner::new(Stream::builder(temp_path()).temporary(true).open().unwrap());

    let mut stream = owner.proxy();

    stream
        .append(
            vec![
                event(
                    "StudentSubscribedToCourse",
                    "hello world!",
                    &["student:3242", "course:523"],
                    0,
                ),
                event("CourseCapacityChanged", "oh, no!", &["course:523"], 0),
                event(
                    "StudentSubscribedToCourse",
                    "goodbye world...",
                    &["student:7642", "course:63"],
                    1,
                ),
            ],
            Condition::new(),
        )
        .unwrap();

    // Select any "StudentSubscribedToCourse" or "CourseCapacityChanged" event
    // that also carries the "course:523" tag.
    let condition = Condition::new().selections([Selection::new([Selector::types_and_tags(
        [
            TypeSelector::new("StudentSubscribedToCourse").unwrap(),
            TypeSelector::new("CourseCapacityChanged").unwrap(),
        ],
        [Tag::new("course:523").unwrap()],
    )])]);

    for event in stream.select(condition) {
        println!("event: {:#?}", event.unwrap());
    }
}
