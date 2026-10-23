//! Build `dictionaries/fst/urdu/joins.fst` from `dictionaries/txt/urdu/joins.tsv`.
//!
//! Run from the `charabia` crate root:
//!   cargo run --bin build_urdu_fst

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use fst::SetBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let tsv_path = manifest_dir.join("dictionaries/txt/urdu/joins.tsv");
    let fst_path = manifest_dir.join("dictionaries/fst/urdu/joins.fst");

    if let Some(parent) = fst_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::open(&tsv_path)?;
    let reader = BufReader::new(file);

    let mut builder = SetBuilder::memory();
    let mut count = 0usize;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let key = line.split('\t').next().unwrap();
        builder.insert(key)?;
        count += 1;
    }

    let bytes = builder.into_set().into_fst().into_inner();
    std::fs::write(&fst_path, bytes)?;

    println!("Built {count} keys into {}", fst_path.display());
    Ok(())
}
