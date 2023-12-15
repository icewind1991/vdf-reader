use std::fs::read_to_string;
use test_case::test_case;
use vdf_reader::entry::Table;
use vdf_reader::Reader;

#[test_case("tests/data/concrete.vmt")]
#[test_case("tests/data/messy.vdf")]
#[test_case("tests/data/DialogConfigOverlay_1280x720.vdf")]
fn test_parse(path: &str) {
    let raw = read_to_string(path).unwrap();
    let mut reader = Reader::from(raw.as_str());
    let parsed = Table::load(&mut reader)
        .map_err(miette::Error::from)
        .expect("failed to parse test data");
    insta::assert_ron_snapshot!(path, parsed);
}
