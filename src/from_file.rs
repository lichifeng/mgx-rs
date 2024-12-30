use crate::{Parser, Record};
use anyhow::{Result, anyhow};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Parse a recorded game file into a `Record` and `Parser`. Game info can be accessed from `Record`
pub fn from_file(file: &str) -> Result<(Record, Parser<Vec<u8>>)> {
    let path = Path::new(file);
    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Get file metadata
    let metadata = fs::metadata(&path)?;
    let filename = path.file_name()
        .ok_or_else(|| anyhow!("Failed to get file name"))?
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert file name to &str"))?
        .to_string();
    let last_modified = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs();

    let mut record = Record::new(filename, buffer.len(), last_modified);
    let mut parser = Parser::<Vec<u8>>::new(buffer).unwrap();
    parser.parse_to(&mut record)?;

    Ok((record, parser))
}
