use crate::parser::Parser;
use crate::record::Record;
use anyhow::Result;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub fn from_file(file: &str) -> Result<(Record, Parser)> {
    let path = Path::new(file);
    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Get file metadata
    let metadata = fs::metadata(&path)?;
    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
    let last_modified = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs();

    let mut record = Record::new(filename, buffer.len(), last_modified);
    let mut parser = Parser::new(buffer).unwrap();
    parser.parse_to(&mut record)?;

    Ok((record, parser))
}
