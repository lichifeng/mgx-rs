use mgx::parser::parse;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::{Instant, UNIX_EPOCH};

fn main() {
    let start_time = Instant::now();

    let path = Path::new("../aoc10a4v4.mgx");

    // Read file into memory as binary data
    let mut file = File::open(&path).expect("Unable to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Unable to read file");

    // Get file metadata
    let metadata = fs::metadata(&path).expect("Unable to get file metadata");
    let filename = path.file_name().unwrap().to_str().unwrap();
    let last_modified = metadata
        .modified()
        .expect("Unable to get modification time")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // Parse the file
    let rec = parse(buffer, filename, last_modified);
    match rec {
        Ok((record, _)) => {
            println!("{:#?}", record);
            println!("ok");
        }
        Err((record, msg)) => {
            println!("{:#?}", record);
            println!("{}", msg);
        }
    }

    let duration = start_time.elapsed();
    println!("Execution time: {} ms", duration.as_millis());
}
