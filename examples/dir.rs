use miette::{Context, IntoDiagnostic, Result};
use std::env::args;
use std::fs::read_to_string;
use std::path::Path;
use vdf_reader::entry::Table;
use vdf_reader::Reader;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let mut success = 0;
    let mut err = Vec::new();
    let dir = args().nth(1).expect("no path provided");
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_str().unwrap_or_default();
            name.ends_with(".vmt") || name.ends_with(".vdf")
        })
    {
        if let Err(e) = try_parse(entry.path()) {
            err.push(e);
            let e = try_parse(entry.path()).unwrap_err();
            println!("{:?}", e);
        } else {
            success += 1;
            println!("{}", entry.path().display());
        }
    }

    println!("successfully parsed {success} files");
    println!("found errors in {} files", err.len());
    for e in err {
        println!("{:?}", e);
    }

    Ok(())
}

fn try_parse(path: &Path) -> Result<Table> {
    let raw = read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to read {}", path.display()))?;
    let mut reader = Reader::from(raw.as_str());
    Table::load(&mut reader).wrap_err_with(|| format!("failed to parse {}", path.display()))
}
