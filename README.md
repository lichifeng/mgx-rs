# mgx
`mgx` is a parser for **Age of Empires II** recorded games.

## Supported version
* AoK(`.mgl`)
* AoC 1.0(`.mgx`)
* AoC 1.0c(`.mgx`)
* Userpatch 1.5 or earlier(`.mgz`)

**Note:** `mgx` doesn't support game records of HD/DE versions.

## Usage (as a binary)
```text
# mgx --help

A parser for Age of Empires II recorded games.

Usage: mgx [OPTIONS] <RECORD_PATH>

Arguments:
  <RECORD_PATH>  Path to the record file. Only AoK(.mgl)/AoC(.mgx)/UP1.5(.mgz) are supported

Options:
  -m <MAP>               Generate a map image as a PNG file. Not rotated.
  -j, --json             Dump game info into a JSON string
      --zh               Use Chinese translations for output
      --header <HEADER>  Dump header section to specified file
      --body <BODY>      Dump body section to specified file
  -h, --help             Print help
  -V, --version          Print version
```

## Usage (as a library)

> Parse from a buffer if you need more control over the parsing.
> Create a `Record` manually and pass it to a `Parser`. Extracted data will persist in the `Record` even if the `Parser` returns an error.
> `mgx::from_file()` will return nothing if any error occurs.

### Parse a file directly
```rust
let filename = "path-to-test-record.mgx";
let (mut rec, parser) = mgx::from_file(filename).unwrap();

// See src/record.rs for more available fields
println!(" Version: {:?}", rec.ver.unwrap());

// Generate a map image as a PNG file.
mgx::draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();

// Encoding of in-game strings is guessed from instructions and may not be correct. `GBK` is used as a fallback.
println!("Encoding: {:?}", rec.detect_encoding().unwrap());

// .convert_encoding() calls .detect_encoding() first.
rec.convert_encoding();

// Some info like civilizations is stored as numeric raw data. `.translate()` converts these to human-readable strings. Only "zh" and "en" are supported now.
rec.translate();

// Dump comprehensive info into a JSON string. Check `null` values before using them.
// This method calls .convert_encoding() first.
println!("{:?}", rec.dump_json().unwrap());
```
### Parse a memory buffer
```rust
use mgx::{Parser, Record};

let mut buffer = Vec::new();

// Prepare filename and last_modified manually
let mut record = Record::new(filename, buffer.len(), last_modified);

// Parsing process won't start until `parse_to()` is called.
let mut parser = Parser::new(buffer).unwrap();
parser.parse_to(&mut record)?;
record.convert_encoding();
record.translate();
```

## References
* <https://github.com/goto-bus-stop/recanalyst.git>
* <https://github.com/happyleavesaoc/aoc-mgz.git>
* <https://github.com/lichifeng/MgxParser.git>
* <https://github.com/stefan-kolb/aoc-mgx-format>