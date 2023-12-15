use miette::{Context, IntoDiagnostic, Result};
use std::env::args;
use std::fs::read_to_string;
use vdf_reader::Reader;

fn main() -> Result<()> {
    let path = args().nth(1).expect("no path provided");
    let raw = read_to_string(path)
        .into_diagnostic()
        .wrap_err("failed to read input")?;
    let reader = Reader::from(raw.as_str());
    for event in reader {
        println!("{:?}", event?);
    }
    Ok(())
}
