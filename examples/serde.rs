use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;
use std::env::args;
use std::fs::read_to_string;
use vdf_reader::from_str;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
enum Material {
    LightmappedGeneric {
        #[serde(rename = "$baseTexture")]
        base_texture: String,
        #[serde(rename = "$bumpmap")]
        bumpmap: String,
        #[serde(rename = "$ssbump")]
        ssbump: bool,
        #[serde(rename = "%keywords")]
        keywords: String,
        #[serde(rename = "$detail")]
        detail: String,
        #[serde(rename = "$detailscale")]
        detailscale: f32,
        #[serde(rename = "$detailblendmode")]
        detailblendmode: i32,
        #[serde(rename = "$detailblendfactor")]
        detailblendfactor: f32,
    },
}

fn main() -> Result<()> {
    let path = args().nth(1).expect("no path provided");
    let raw = read_to_string(path)
        .into_diagnostic()
        .wrap_err("failed to read input")?;
    let material: Material = from_str(&raw)?;
    dbg!(material);
    Ok(())
}
