mod fixtures;

use eventric_stream::{
    error::Error,
    stream::{
        Stream,
        append::Append,
        iterate::Iterate,
    },
    temp_path,
};

// =================================================================================================
// Properties
// =================================================================================================

// Stream::len

#[test]
fn stream_len() -> Result<(), Error> {
    let path = eventric_stream::temp_path();

    // Property after multiple length changing-operations

    {
        let mut stream = Stream::builder(&path).open()?;

        assert_eq!(stream.len(), 0);

        stream.append(fixtures::event("one", "id_one", &[], 0), None)?;

        assert_eq!(stream.len(), 1);

        stream.append(fixtures::event("two", "id_two", &[], 0), None)?;
        stream.append(fixtures::event("three", "id_three", &[], 0), None)?;
        stream.append(fixtures::event("four", "id_four", &[], 0), None)?;

        assert_eq!(stream.len(), 4);

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 11);
    }

    // Property after re-open (persistence) and length-changing operation

    {
        let mut stream = Stream::builder(&path).temporary(true).open()?;

        assert_eq!(stream.len(), 11);

        stream.append(fixtures::events()?, None)?;

        assert_eq!(stream.len(), 18);
    }

    Ok(())
}

#[test]
fn len_empty_stream() -> Result<(), Error> {
    let stream = fixtures::stream()?;

    assert_eq!(stream.len(), 0, "New stream should have length 0");

    Ok(())
}

#[test]
fn len_after_single_append() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;

    assert_eq!(
        stream.len(),
        1,
        "Stream should have length 1 after appending 1 event"
    );

    Ok(())
}

#[test]
fn len_after_multiple_appends() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;
    stream.append([fixtures::event("event2", "Event", &[], 0)?], None)?;
    stream.append([fixtures::event("event3", "Event", &[], 0)?], None)?;

    assert_eq!(
        stream.len(),
        3,
        "Stream should have length 3 after 3 appends"
    );

    Ok(())
}

#[test]
fn len_after_batch_append() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(fixtures::events()?, None)?;

    assert_eq!(
        stream.len(),
        7,
        "Stream should have length 7 after batch append"
    );

    Ok(())
}

#[test]
fn len_increases_monotonically() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    assert_eq!(stream.len(), 0);

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 1);

    stream.append([fixtures::event("event2", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 2);

    stream.append(fixtures::events()?, None)?;
    assert_eq!(stream.len(), 9);

    Ok(())
}

#[test]
fn len_after_multiple_batches() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(fixtures::events()?, None)?;
    assert_eq!(stream.len(), 7);

    stream.append(fixtures::events()?, None)?;
    assert_eq!(stream.len(), 14);

    stream.append(fixtures::events()?, None)?;
    assert_eq!(stream.len(), 21);

    Ok(())
}

#[test]
fn len_with_different_event_types() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "EventA", &[], 0)?], None)?;
    stream.append([fixtures::event("event2", "EventB", &[], 1)?], None)?;
    stream.append([fixtures::event("event3", "EventC", &["tag:1"], 2)?], None)?;

    assert_eq!(
        stream.len(),
        3,
        "Length should count all events regardless of type"
    );

    Ok(())
}

#[test]
fn len_persists_across_reopens() -> Result<(), Error> {
    let path = temp_path();

    {
        let mut stream = Stream::builder(&path).open()?;
        stream.append(fixtures::events()?, None)?;
        assert_eq!(stream.len(), 7);
    }

    {
        let stream = Stream::builder(&path).open()?;
        assert_eq!(stream.len(), 7, "Length should persist after reopening");
    }

    std::fs::remove_dir_all(&path).unwrap();

    Ok(())
}

// Stream::is_empty

#[test]
fn is_empty_new_stream() -> Result<(), Error> {
    let stream = fixtures::stream()?;

    assert!(stream.is_empty(), "New stream should be empty");

    Ok(())
}

#[test]
fn is_empty_after_append() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;

    assert!(
        !stream.is_empty(),
        "Stream should not be empty after append"
    );

    Ok(())
}

#[test]
fn is_empty_false_with_one_event() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;

    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 1);

    Ok(())
}

#[test]
fn is_empty_false_with_many_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(fixtures::events()?, None)?;

    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 7);

    Ok(())
}

#[test]
fn is_empty_consistency_with_len() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Empty stream
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);

    // After first append
    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;
    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 1);

    // After more appends
    stream.append(fixtures::events()?, None)?;
    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 8);

    Ok(())
}

#[test]
fn is_empty_persists_across_reopens() -> Result<(), Error> {
    let path = temp_path();

    {
        let mut stream = Stream::builder(&path).open()?;
        assert!(stream.is_empty(), "New stream should be empty");

        stream.append(fixtures::events()?, None)?;
        assert!(
            !stream.is_empty(),
            "Stream should not be empty after append"
        );
    }

    {
        let stream = Stream::builder(&path).open()?;
        assert!(
            !stream.is_empty(),
            "is_empty should persist after reopening"
        );
    }

    std::fs::remove_dir_all(&path).unwrap();

    Ok(())
}

// Combined len and is_empty tests

#[allow(clippy::len_zero)]
#[test]
fn len_and_is_empty_relationship() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    // Initially empty
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.is_empty(), stream.len() == 0);

    // After appending
    for i in 1..=10 {
        stream.append(
            [fixtures::event(&format!("event{i}"), "Event", &[], 0)?],
            None,
        )?;
        assert_eq!(stream.is_empty(), stream.len() == 0);
        assert!(!stream.is_empty());
        assert_eq!(stream.len(), i);
    }

    Ok(())
}

#[test]
fn len_with_mixed_operations() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    assert_eq!(stream.len(), 0);

    stream.append(fixtures::events()?, None)?;
    assert_eq!(stream.len(), 7);

    stream.append([fixtures::event("single", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 8);

    Ok(())
}

#[test]
fn properties_unchanged_by_queries() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(fixtures::events()?, None)?;
    let len_before = stream.len();
    let is_empty_before = stream.is_empty();

    // Perform some queries (they shouldn't affect length or emptiness)
    let _events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;

    assert_eq!(stream.len(), len_before);
    assert_eq!(stream.is_empty(), is_empty_before);

    Ok(())
}

#[test]
fn properties_with_tagged_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(
        vec![
            fixtures::event("event1", "Event", &["tag:a", "tag:b"], 0)?,
            fixtures::event("event2", "Event", &["tag:c"], 0)?,
            fixtures::event("event3", "Event", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(stream.len(), 3);
    assert!(!stream.is_empty());

    Ok(())
}

#[test]
fn properties_with_versioned_events() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(
        vec![
            fixtures::event("v0", "Event", &[], 0)?,
            fixtures::event("v1", "Event", &[], 1)?,
            fixtures::event("v2", "Event", &[], 2)?,
            fixtures::event("v0_again", "Event", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(stream.len(), 4, "All versions should be counted");
    assert!(!stream.is_empty());

    Ok(())
}

#[test]
fn properties_stable_across_multiple_reads() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append(fixtures::events()?, None)?;

    // Read properties multiple times
    for _ in 0..10 {
        assert_eq!(stream.len(), 7);
        assert!(!stream.is_empty());
    }

    Ok(())
}

#[test]
fn len_increments_correctly_with_interleaved_operations() -> Result<(), Error> {
    let mut stream = fixtures::stream()?;

    stream.append([fixtures::event("event1", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 1);

    let _events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(stream.len(), 1, "Iteration shouldn't change length");

    stream.append([fixtures::event("event2", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 2);

    Ok(())
}
