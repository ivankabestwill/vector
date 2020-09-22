use super::*;
use std::{fs, io::Read, path::Path};

const SUFFICIENTLY_COMPLEX: &str =
    r#"regular."quoted"."quoted but spaces"."quoted.but.periods".lookup[0].nested_lookup[0][0]"#;
lazy_static::lazy_static! {
    static ref SUFFICIENTLY_DECOMPOSED: [Segment; 9] = [
        Segment::field(r#"regular"#.to_string()),
        Segment::field(r#""quoted""#.to_string()),
        Segment::field(r#""quoted but spaces""#.to_string()),
        Segment::field(r#""quoted.but.periods""#.to_string()),
        Segment::field(r#"lookup"#.to_string()),
        Segment::index(0),
        Segment::field(r#"nested_lookup"#.to_string()),
        Segment::index(0),
        Segment::index(0),
    ];
}

#[test]
fn simple() {
    crate::test_util::trace_init();
    let input = "some_key";
    let lookup = Lookup::try_from(input).unwrap();
    assert_eq!(lookup[0], Segment::field(String::from("some_key")));
    assert_eq!(lookup.to_string(), input);
}

#[test]
fn array() {
    crate::test_util::trace_init();
    let input = "foo[0]";
    let lookup = Lookup::try_from(input).unwrap();
    assert_eq!(lookup[0], Segment::field(String::from("foo")));
    assert_eq!(lookup[1], Segment::index(0));
    assert_eq!(lookup.to_string(), input);
}

#[test]
fn to_string() {
    crate::test_util::trace_init();
    let input = SUFFICIENTLY_COMPLEX;
    let lookup = Lookup::try_from(input).unwrap();
    assert_eq!(lookup.to_string(), input);
}

#[test]
fn impl_index_ranges() {
    crate::test_util::trace_init();
    let lookup = Lookup::try_from(SUFFICIENTLY_COMPLEX).unwrap();

    // This test is primarily to ensure certain interfaces exist and weren't disturbed.
    assert_eq!(lookup[..], SUFFICIENTLY_DECOMPOSED[..]);
    assert_eq!(lookup[..4], SUFFICIENTLY_DECOMPOSED[..4]);
    assert_eq!(lookup[..=4], SUFFICIENTLY_DECOMPOSED[..=4]);
    assert_eq!(lookup[2..], SUFFICIENTLY_DECOMPOSED[2..]);
}

#[test]
fn impl_index_usize() {
    crate::test_util::trace_init();
    let lookup = Lookup::try_from(SUFFICIENTLY_COMPLEX).unwrap();

    for i in 0..SUFFICIENTLY_DECOMPOSED.len() {
        assert_eq!(lookup[i], SUFFICIENTLY_DECOMPOSED[i])
    }
}

#[test]
fn impl_index_mut_index_mut() {
    crate::test_util::trace_init();
    let mut lookup = Lookup::try_from(SUFFICIENTLY_COMPLEX).unwrap();

    for i in 0..SUFFICIENTLY_DECOMPOSED.len() {
        let x = &mut lookup[i]; // Make sure we force a mutable borrow!
        assert_eq!(*x, SUFFICIENTLY_DECOMPOSED[i])
    }
}

#[test]
fn iter() {
    crate::test_util::trace_init();
    let lookup = Lookup::try_from(SUFFICIENTLY_COMPLEX).unwrap();

    let mut iter = lookup.iter();
    for (index, expected) in SUFFICIENTLY_DECOMPOSED.iter().enumerate() {
        let parsed = iter.next().expect(&format!(
            "Expected at index {}: {:?}, got None.",
            index, expected
        ));
        assert_eq!(expected, parsed, "Failed at {}", index);
    }
}

#[test]
fn into_iter() {
    crate::test_util::trace_init();
    let lookup = Lookup::try_from(SUFFICIENTLY_COMPLEX).unwrap();
    let mut iter = lookup.into_iter();
    for (index, expected) in SUFFICIENTLY_DECOMPOSED.iter().cloned().enumerate() {
        let parsed = iter.next().expect(&format!(
            "Expected at index {}: {:?}, got None.",
            index, expected
        ));
        assert_eq!(expected, parsed, "Failed at {}", index);
    }
}

fn parse_artifact(path: impl AsRef<Path>) -> std::io::Result<String> {
    let mut test_file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut buf = Vec::new();
    test_file.read_to_end(&mut buf)?;
    let string = String::from_utf8(buf).unwrap();
    Ok(string)
}

// This test iterates over the `tests/data/fixtures/lookup` folder and ensures the lookup parsed,
// then turned into a string again is the same.
#[test]
fn lookup_to_string() {
    crate::test_util::trace_init();
    const FIXTURE_ROOT: &str = "tests/data/fixtures/lookup";

    trace!(?FIXTURE_ROOT, "Opening.");
    std::fs::read_dir(FIXTURE_ROOT)
        .unwrap()
        .for_each(|fixture_file| match fixture_file {
            Ok(fixture_file) => {
                let path = fixture_file.path();
                tracing::trace!(?path, "Opening.");
                let buf = parse_artifact(&path).unwrap();
                let lookup = Lookup::try_from(buf.clone()).unwrap();
                tracing::trace!(
                        ?path,
                        ?lookup,
                        ?buf,
                        "Asserting equal."
                    );
                assert_eq!(lookup.to_string(), buf);
            }
            _ => panic!("This test should never read Err'ing test fixtures."),
        });
}