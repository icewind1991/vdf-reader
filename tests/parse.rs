use std::fs::read_to_string;
use test_case::test_case;
use vdf_reader::entry::Table;
use vdf_reader::Reader;

#[test_case("tests/data/concrete.vmt")]
fn test_parse(path: &str) {
    let raw = read_to_string(path).unwrap();
    let mut reader = Reader::from(raw.as_str());
    let parsed = Table::load(&mut reader).expect("failed to parse test data");
    insta::assert_ron_snapshot!(parsed);
}
