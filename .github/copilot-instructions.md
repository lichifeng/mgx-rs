# mgx-rs Copilot Instructions

## Project Overview
`mgx-rs` is a Rust parser for Age of Empires II recorded game files (`.mgl`, `.mgx`, `.mgz`). It extracts game metadata, player information, chat logs, and generates map visualizations from binary replay files. The project supports AoK, AoC 1.0/1.0c, and UserPatch 1.5 versions (but NOT HD/DE editions).

## Architecture

### Core Components
- **`Parser`** (`src/parser.rs`): Handles binary decompression and splits the replay into header/body cursors. Zlib-decompresses header data and calculates MD5 hashes.
- **`Record`** (`src/record.rs`): Data container for all extracted game info. Most fields are `Option<T>` since parsing may be incomplete due to format variations or errors.
- **`StreamCursor`** (`src/cursor.rs`): Custom buffer abstraction that tracks position within a data stream offset from source. Provides `get_*` methods for reading typed data and `mov/seek` for navigation.
- **`body_parser`** (`src/body_parser.rs`): Parses game commands (operations like `OP_SYNC`, `OP_COMMAND`, `OP_CHAT`) to extract duration, resignations, age advancements, and chat messages.

### Data Flow
1. File → `Parser::new()` → Decompress header + separate body
2. `Parser::parse_to(&mut Record)` → Extract metadata from header + parse body operations
3. `Record::convert_encoding()` → Detect and decode text fields (GBK/UTF-8/Big5/etc.)
4. `Record::translate()` → Convert numeric IDs to human-readable strings (civs, maps, etc.)
5. `Record::dump_json()` → Serialize to JSON (calls `convert_encoding()` first)

## Key Patterns

### Error Handling
- Use `anyhow::Result` for all fallible operations
- The `val!()` macro unwraps `Option<T>` or bails with debug info: `val!(record.versave)`
- Parsing continues on errors when possible—check for `None` fields in `Record`
- Debug assertions (`debug_assert!`, `#[cfg(debug_assertions)]`) validate assumptions during development

### Binary Parsing with StreamCursor
```rust
let h = &mut parser.header;
h.mov(8);                    // Move cursor 8 bytes forward
let version = h.get_f32();    // Read f32 at current position
let value = h.peek_u32();     // Read without advancing cursor
```
Common methods: `get_u8/i8/u16/i16/u32/i32/f32`, `get_str_fixed`, `get_bytes`, `peek_*`, `mov`, `seek`, `tell`, `remain`

### Version Detection
Version is determined by checking `verraw` header bytes and `verlog` body values. See `src/parser.rs` lines 75-110 for version detection logic. Handle version-specific behavior with match statements on `r.ver`.

### Text Encoding
Game strings use region-specific encodings detected via pattern matching in `instructions_raw` (see `ENCODING_MAP` in `src/render.rs`). Always call `convert_encoding()` before accessing decoded text fields. Fallback is GBK.

### Translation System
Numeric IDs are translated using the `trans!()` macro with PHF maps in `src/translations/{en,zh}.rs`. Example:
```rust
self.mapname = trans!(self.mapid, lang, MAP_NAMES_TRANS);
```

## Development Workflows

### Running Tests
```bash
cargo test                    # Run all tests (includes parsing test replays)
cargo test --release          # Faster for large replay files
```
Test files are in `tests/recs/`. Tests verify GUID calculation, version detection, duration parsing, and matchup extraction.

### Building & Running
```bash
cargo build --release
cargo run --release -- tests/recs/aoc10a_1v1_with_winner.mgx -j --zh
cargo run -- <replay_file> -m map.png -j --zh  # Generate map + JSON output
```

### Debugging Binary Formats
```bash
mgx <replay_file> --header header.bin --body body.bin  # Dump raw sections
```
Use hex editors to inspect dumped sections. Body format: alternating operation type (4 bytes) + operation data.

### Code Formatting
Project uses custom rustfmt config (`rustfmt.toml`): `max_width = 120`, `use_small_heuristics = "Max"`.

## Common Tasks

### Adding Version Support
1. Update version detection logic in `src/parser.rs` (check `verraw` and `verlog`)
2. Add enum variant to `Version` in `src/record.rs`
3. Handle version-specific parsing differences in `Parser::parse_to()`
4. Add test file to `tests/recs/` and test case to `tests/parser.rs`

### Extending Record Fields
1. Add field to `Record` struct in `src/record.rs` (use `Option<T>` for nullable data)
2. Add `#[serde(skip)]` for raw byte fields (e.g., `name_raw`, `instructions_raw`)
3. Extract field in `Parser::parse_to()` or `parse_body()`
4. Add conversion logic to `convert_encoding()` or `translate()` if needed

### Adding Translation Keys
Add entries to `src/translations/{en,zh}.rs` PHF maps. Keys are numeric IDs, values are localized strings.

## Important Notes
- Body parsing uses fixed operation types: `0x01` (command), `0x02` (sync/time), `0x03` (viewlock), `0x04` (chat)
- Time is tracked in milliseconds via `OP_SYNC` operations with `time_delta` values
- Player slots are 0-8 (slot 0 is GAIA). Check `player.isvalid()` before using player data
- Map rendering (`draw_map.rs`) rotates 45° counterclockwise and scales height to 50% for isometric view
- GUID calculation (`guid.rs`) hashes early movement commands for replay identification
- **Do not** add support for HD/DE editions—different binary formats entirely
