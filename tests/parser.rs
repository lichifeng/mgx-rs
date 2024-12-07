use anyhow::{bail, Result};
use mgx::parser::Parser;
use mgx::record::{Record, Version};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

fn load_file(path: &str) -> (Vec<u8>, String, u64) {
    // Read file into memory as binary data
    let path = Path::new(path);
    let mut file = File::open(&path).expect("Unable to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Unable to read file");

    // Get file metadata
    let metadata = fs::metadata(&path).expect("Unable to get file metadata");
    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
    let last_modified = metadata
        .modified()
        .expect("Unable to get modification time")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    (buffer, filename, last_modified)
}

fn try_parse(path: &str) -> Result<Record> {
    let base_dir = String::from("tests/recs/");
    let path = base_dir + path;

    // Load the file
    let (buffer, filename, last_modified) = load_file(&path);

    // Create a record
    let mut record = Record::new(filename, buffer.len(), last_modified);

    // Parse the file
    let mut parser = Parser::new(buffer).unwrap();
    match parser.parse_to(&mut record) {
        Ok(_) => match record.ver {
            Some(Version::DE) | Some(Version::HD) => {}
            _ => {
                let map_name = format!("{}.png", path);
                parser.draw_map(&map_name)?;
                if Path::new(&map_name).exists() {
                    fs::remove_file(map_name)?;
                } else {
                    bail!("testmap was not generated");
                }
            }
        },
        _ => {}
    }

    Ok(record)
}

#[test]
fn aok_test() {
    let retval = try_parse("aok_4v4_fast.mgl").unwrap(); // This guid is from MgxParser
    println!("aok versave: {:?}", retval.versave);
    assert_eq!(retval.ver, Some(Version::AoK));
}

#[test]
fn aoc10_test() {
    let retval = try_parse("28083e6497dc1a0a3f8ca3a54c2622c2.mgx").unwrap(); // This guid is from MgxParser
    println!("aoc10 versave: {:?}", retval.versave);
    assert_eq!(retval.ver, Some(Version::AoC10a));
}

#[test]
fn aoc10c_test() {
    let retval = try_parse("aoc-1.0c.mgx").unwrap();
    println!("aoc10c versave: {:?}", retval.versave);
    assert_eq!(retval.ver, Some(Version::AoC10c));
}

#[test]
fn up15_test() {
    let retval = try_parse("up1.5.mgz").unwrap();
    println!("up15 versave: {:?}", retval.versave);
    assert_eq!(retval.ver, Some(Version::UP15));
}

#[test]
fn up14_scenario_test() {
    let retval = try_parse("scenario-with-messages.mgz").unwrap();
    assert!(retval.verscenario.unwrap() - 1.22 < 0.0001);
    assert_eq!(retval.ver, Some(Version::UP14));
}

#[test]
fn de63_test() {
    let retval = try_parse("de-63.0.aoe2record").unwrap();
    println!("up15 versave: {:?}", retval.versave);
    assert_eq!(retval.ver, Some(Version::DE));
}

#[test]
fn ai_test() {
    let retval = try_parse("1v7_hardest_spain_aoc10.mgx").unwrap();
    assert_eq!(retval.include_ai, Some(true));
    println!("speed: {:?}", retval.speed);
    println!("map: {:?}, {:?}", retval.mapx, retval.mapy);
}
