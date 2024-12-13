use clap::Parser;
use mgx::{draw_map, from_file};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the record file. Only AoK(.mgl)/AoC(.mgx)/UP1.5(.mgz) are supported.
    record_path: PathBuf,

    /// Generate a map image as a .png image.   
    /// Rotated 45° counterclockwise and change height to 50% to get a in-game look.
    #[arg(short = 'm')]
    map: Option<PathBuf>,

    /// Dump record information into a JSON string.
    #[arg(short = 'j', long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let (mut rec, parser) = from_file(cli.record_path.to_str().unwrap()).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    // Generate map if -m option is provided
    if let Some(map_path) = cli.map {
        draw_map(&rec, &parser, map_path.to_str().unwrap()).expect("Failed to generate a map");
    }

    // Print JSON if -j/--json flag is set
    if cli.json {
        rec.translate("en");
        if let Ok(json) = rec.dump_json() {
            println!("{}", json);
        } else {
            eprintln!("Failed to generate JSON output");
        }
    } else {
        println!("Filename: {}", rec.filename);
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
    }
}
