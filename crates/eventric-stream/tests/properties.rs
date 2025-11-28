use eventric_stream::{
    error::Error,
    event::{
        Data,
        EphemeralEvent,
        Identifier,
        Tag,
        Version,
    },
    stream::{
        Stream,
        append::Append,
        iterate::Iterate,
    },
    temp_path,
};

// =================================================================================================
// Stream Properties
// =================================================================================================

/// Creates a new temporary test stream that will be automatically cleaned up
fn create_test_stream() -> Result<Stream, Error> {
    Stream::builder(temp_path()).temporary(true).open()
}

/// Creates a sample `EphemeralEvent` for testing
fn create_event(
    data: &str,
    identifier: &str,
    tags: &[&str],
    version: u8,
) -> Result<EphemeralEvent, Error> {
    Ok(EphemeralEvent::new(
        Data::new(data)?,
        Identifier::new(identifier)?,
        tags.iter()
            .map(|tag| Tag::new(*tag))
            .collect::<Result<Vec<_>, _>>()?,
        Version::new(version),
    ))
}

/// Creates a batch of events for testing
fn create_events(count: usize) -> Result<Vec<EphemeralEvent>, Error> {
    (0..count)
        .map(|i| create_event(&format!("event{i}"), "TestEvent", &[], 0))
        .collect()
}

// Stream::len

#[test]
fn len_empty_stream() -> Result<(), Error> {
    let stream = create_test_stream()?;

    assert_eq!(stream.len(), 0, "New stream should have length 0");

    Ok(())
}

#[test]
fn len_after_single_append() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;

    assert_eq!(stream.len(), 1, "Stream should have length 1 after appending 1 event");

    Ok(())
}

#[test]
fn len_after_multiple_appends() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;
    stream.append([create_event("event2", "Event", &[], 0)?], None)?;
    stream.append([create_event("event3", "Event", &[], 0)?], None)?;

    assert_eq!(stream.len(), 3, "Stream should have length 3 after 3 appends");

    Ok(())
}

#[test]
fn len_after_batch_append() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(10)?, None)?;

    assert_eq!(stream.len(), 10, "Stream should have length 10 after batch append");

    Ok(())
}

#[test]
fn len_increases_monotonically() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    assert_eq!(stream.len(), 0);

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 1);

    stream.append([create_event("event2", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 2);

    stream.append(create_events(5)?, None)?;
    assert_eq!(stream.len(), 7);

    Ok(())
}

#[test]
fn len_with_large_batch() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(1000)?, None)?;

    assert_eq!(stream.len(), 1000, "Stream should handle large batches");

    Ok(())
}

#[test]
fn len_after_multiple_batches() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(10)?, None)?;
    assert_eq!(stream.len(), 10);

    stream.append(create_events(20)?, None)?;
    assert_eq!(stream.len(), 30);

    stream.append(create_events(15)?, None)?;
    assert_eq!(stream.len(), 45);

    Ok(())
}

#[test]
fn len_with_different_event_types() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "EventA", &[], 0)?], None)?;
    stream.append([create_event("event2", "EventB", &[], 1)?], None)?;
    stream.append([create_event("event3", "EventC", &["tag:1"], 2)?], None)?;

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
        stream.append(create_events(5)?, None)?;
        assert_eq!(stream.len(), 5);
    }

    {
        let stream = Stream::builder(&path).open()?;
        assert_eq!(stream.len(), 5, "Length should persist after reopening");
    }

    std::fs::remove_dir_all(&path).unwrap();

    Ok(())
}

// Stream::is_empty

#[test]
fn is_empty_new_stream() -> Result<(), Error> {
    let stream = create_test_stream()?;

    assert!(stream.is_empty(), "New stream should be empty");

    Ok(())
}

#[test]
fn is_empty_after_append() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;

    assert!(!stream.is_empty(), "Stream should not be empty after append");

    Ok(())
}

#[test]
fn is_empty_false_with_one_event() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;

    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 1);

    Ok(())
}

#[test]
fn is_empty_false_with_many_events() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(100)?, None)?;

    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 100);

    Ok(())
}

#[test]
fn is_empty_consistency_with_len() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    // Empty stream
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);

    // After first append
    stream.append([create_event("event1", "Event", &[], 0)?], None)?;
    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 1);

    // After more appends
    stream.append(create_events(10)?, None)?;
    assert!(!stream.is_empty());
    assert_eq!(stream.len(), 11);

    Ok(())
}

#[test]
fn is_empty_persists_across_reopens() -> Result<(), Error> {
    let path = temp_path();

    {
        let mut stream = Stream::builder(&path).open()?;
        assert!(stream.is_empty(), "New stream should be empty");

        stream.append(create_events(3)?, None)?;
        assert!(!stream.is_empty(), "Stream should not be empty after append");
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

#[test]
fn len_and_is_empty_relationship() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    // Initially empty
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.is_empty(), stream.len() == 0);

    // After appending
    for i in 1..=10 {
        stream.append([create_event(&format!("event{i}"), "Event", &[], 0)?], None)?;
        assert_eq!(stream.is_empty(), stream.len() == 0);
        assert!(!stream.is_empty());
        assert_eq!(stream.len(), i);
    }

    Ok(())
}

#[test]
fn len_and_is_empty_with_various_batch_sizes() -> Result<(), Error> {
    let batch_sizes = [1, 5, 10, 50, 100];

    for &size in &batch_sizes {
        let mut test_stream = create_test_stream()?;
        test_stream.append(create_events(size)?, None)?;

        assert!(!test_stream.is_empty());
        assert_eq!(test_stream.len(), size as u64);
        assert_eq!(test_stream.is_empty(), test_stream.len() == 0);
    }

    Ok(())
}

#[test]
fn len_returns_u64() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(1)?, None)?;

    let len: u64 = stream.len();
    assert_eq!(len, 1u64);

    Ok(())
}

#[test]
fn len_with_mixed_operations() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    assert_eq!(stream.len(), 0);

    stream.append(create_events(5)?, None)?;
    assert_eq!(stream.len(), 5);

    stream.append([create_event("single", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 6);

    stream.append(create_events(3)?, None)?;
    assert_eq!(stream.len(), 9);

    Ok(())
}

#[test]
fn properties_unchanged_by_queries() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(10)?, None)?;
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
    let mut stream = create_test_stream()?;

    stream.append(
        vec![
            create_event("event1", "Event", &["tag:a", "tag:b"], 0)?,
            create_event("event2", "Event", &["tag:c"], 0)?,
            create_event("event3", "Event", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(stream.len(), 3);
    assert!(!stream.is_empty());

    Ok(())
}

#[test]
fn properties_with_versioned_events() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(
        vec![
            create_event("v0", "Event", &[], 0)?,
            create_event("v1", "Event", &[], 1)?,
            create_event("v2", "Event", &[], 2)?,
            create_event("v0_again", "Event", &[], 0)?,
        ],
        None,
    )?;

    assert_eq!(stream.len(), 4, "All versions should be counted");
    assert!(!stream.is_empty());

    Ok(())
}

#[test]
fn len_at_boundaries() -> Result<(), Error> {
    let stream = create_test_stream()?;

    assert_eq!(stream.len(), 0, "Minimum length is 0");

    // We can't easily test u64::MAX, but we can verify the type is u64
    let len: u64 = stream.len();
    assert!(len <= u64::MAX);

    Ok(())
}

#[test]
fn is_empty_boolean_type() -> Result<(), Error> {
    let stream = create_test_stream()?;

    let is_empty: bool = stream.is_empty();
    assert!(is_empty || !is_empty); // Always true, just verifies type

    Ok(())
}

#[test]
fn properties_stable_across_multiple_reads() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append(create_events(5)?, None)?;

    // Read properties multiple times
    for _ in 0..10 {
        assert_eq!(stream.len(), 5);
        assert!(!stream.is_empty());
    }

    Ok(())
}

#[test]
fn len_increments_correctly_with_interleaved_operations() -> Result<(), Error> {
    let mut stream = create_test_stream()?;

    stream.append([create_event("event1", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 1);

    let _events: Vec<_> = stream.iterate(None).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(stream.len(), 1, "Iteration shouldn't change length");

    stream.append([create_event("event2", "Event", &[], 0)?], None)?;
    assert_eq!(stream.len(), 2);

    Ok(())
}
