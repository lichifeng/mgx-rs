use clap::Parser;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the record file. Only AoK(.mgl)/AoC(.mgx)/UP1.5(.mgz) are supported.
    record_path: PathBuf,

    /// Generate a map image as a .png image.   
    /// Rotated 45° counterclockwise and change height to 50% to get a in-game look.
    #[arg(short = 'm')]
    map: Option<PathBuf>,

    /// Dump game info into a JSON string.
    #[arg(short = 'j', long)]
    json: bool,

    /// Use Chinese language for output
    #[arg(long)]
    zh: bool,

    /// Dump header section to specified file
    #[arg(long)]
    header: Option<PathBuf>,

    /// Dump body section to specified file
    #[arg(long)]
    body: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let path = Path::new(cli.record_path.to_str().unwrap());
    let mut file = File::open(&path).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {}", path.to_string_lossy(), e);
        std::process::exit(1);
    });
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    // Get file metadata
    let metadata = fs::metadata(&path).unwrap();
    let filename = path.file_name().unwrap().to_string_lossy();
    let last_modified = metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis();

    let mut rec = mgx::Record::new(filename.into_owned(), buffer.len(), last_modified);
    let mut parser = mgx::Parser::new(buffer).unwrap();

    if let Some(header_path) = cli.header {
        parser.dump_header(header_path.to_str().unwrap()).unwrap_or_else(|e| {
            eprintln!("Error dumping header: {}", e);
            std::process::exit(1);
        });
    }

    if let Some(body_path) = cli.body {
        parser.dump_body(body_path.to_str().unwrap()).unwrap_or_else(|e| {
            eprintln!("Error dumping body: {}", e);
            std::process::exit(1);
        });
    }

    parser.parse_to(&mut rec).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    if let Some(map_path) = cli.map {
        mgx::draw_map(&rec, &parser, map_path.to_str().unwrap()).unwrap_or_else(|e| {
            eprintln!("Error: {}. Remove -m to get available data.", e);
            std::process::exit(1);
        });
    }

    if cli.zh {
        rec.translate("zh");
    } else {
        rec.translate("en");
    }

    // Print JSON if -j/--json flag is set
    if cli.json {
        if let Ok(json) = rec.dump_json() {
            println!("{}", json);
        } else {
            eprintln!("Failed to generate JSON output");
        }
    } else {
        println!("Filename: {}", rec.filename);
        println!("   Speed: {}", rec.speed.unwrap_or("N/A".to_string()));
        println!(" Version: {:?}", rec.ver.unwrap());
        println!(
            " Matchup: {}",
            rec.matchup.unwrap_or_default().iter().map(|t| t.to_string()).collect::<Vec<_>>().join("v")
        );
        println!(
            "Duration: {:02}:{:02}:{:02}",
            rec.duration / 1000 / 60 / 60,
            (rec.duration / 1000 / 60) % 60,
            (rec.duration / 1000) % 60
        );
        println!("    GUID: {}", rec.guid.unwrap_or("N/A".to_string()));
    }
}
