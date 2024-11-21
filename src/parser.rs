use crate::cursor::StreamCursor;
use crate::drawmap::draw_map;
use crate::record::{Chat, Record};
use flate2::read::ZlibDecoder;
use flate2::Decompress;
use std::io::Read;
use std::ops::Index;
use std::process::abort;

pub fn parse(buffer: Vec<u8>, filename: &str, lastmod: u64) -> Result<(Record, String), (Record, String)> {
    let mut r = Record::new();

    r.filename = Some(filename.to_string());
    r.filesize = Some(buffer.len());
    r.lastmod = Some(lastmod);

    r.debug.headerend = u32::from_le_bytes(buffer[0..4].try_into().expect("Invalid length"));
    r.debug.nextpos = u32::from_le_bytes(buffer[4..8].try_into().expect("Invalid length"));
    r.debug.headerstart = if r.debug.nextpos < buffer.len() as u32 {
        8
    } else {
        r.vercode = Some("aok".to_string());
        4
    };

    let mut header_buffer = Vec::new();
    let rawheader = &buffer[r.debug.headerstart as usize..];
    let mut decompressor = ZlibDecoder::new_with_decompress(rawheader, Decompress::new(false));
    decompressor.read_to_end(&mut header_buffer).expect("Unable to decompress header data");
    r.debug.headerlen = header_buffer.len();

    let mut h = StreamCursor::new(header_buffer, 0); // header cursor
    let mut b = StreamCursor::new(buffer, r.debug.headerend as usize); // body cursor

    // TODO: Detect version
    // https://github.com/lichifeng/MgxParser/blob/master/src/analyzers/default/subproc_detectversion.cc
    r.verstr = h.get_cstring(Some(8));
    r.versave = h.get_f32();
    if -1.0 == r.versave.unwrap() {
        // TODO: need test
        r.versave2 = h.get_u32();
    }
    r.debug.aipos = h.tell();
    r.include_ai = h.get_bool(4);

    if r.include_ai.unwrap() {
        // TODO: Parse AI data
    }

    // Replay
    h.mov(12);
    r.speed = h.get_u32();
    h.mov(29);
    r.recorder = h.get_u16();
    r.totalplayers = h.get_u8();
    // TODO
    r.instantbuild = h.get_bool(1);
    r.enablecheats = h.get_bool(1);
    h.mov(2 + 58);
    // TODO

    // Map
    r.mapx = h.get_i32();
    r.mapy = h.get_i32();
    if r.mapx.unwrap() < 0 || r.mapy.unwrap() < 0 {
        return Err((r, "Map size is negative".to_string()));
    } else if r.mapx.unwrap() > 255 || r.mapy.unwrap() > 255 {
        return Err((r, "Map size is too large".to_string()));
    } else if r.mapx.unwrap() != r.mapy.unwrap() {
        return Err((r, "Map is not square".to_string()));
    }
    // TODO handle exceptions
    let num_mapzones = h.get_i32().unwrap();
    let map_bits: isize = (r.mapx.unwrap() * r.mapy.unwrap()) as isize;
    for _ in 0..num_mapzones as usize {
        // TODO
        let num_floats = h.get_i32().unwrap();
        h.mov(num_floats as isize * 4 + 4);
    }
    r.nofog = h.get_bool(1);
    h.mov(1);

    r.debug.mappos = h.tell();

    let peekpos = h.tell() + 7 * map_bits as usize;
    let checkval: u32 = u32::from_le_bytes(h.data()[peekpos..peekpos + 4].try_into().expect("checkval failed"));
    // TODO: condition for DE
    let maptile_type: isize = if h.peek_u8().unwrap() == 255 { 4 } else { 2 };
    h.mov(map_bits * maptile_type);
    let num_data = h.get_i32().unwrap() as isize;
    h.mov(4 + num_data * 4);
    for _ in 0..num_data {
        let num_obstructions = h.get_i32().unwrap() as isize;
        h.mov(num_obstructions * 8);
    }
    let visibility_mapsize0 = h.get_i32().unwrap() as isize;
    let visibility_mapsize1 = h.get_i32().unwrap() as isize;
    h.mov(visibility_mapsize0 * visibility_mapsize1 * 4);
    r.restoretime = h.get_u32();
    let num_particles = h.get_u32().unwrap();
    h.mov(27 * num_particles as isize + 4);

    r.debug.initpos = h.tell();

    // Find trigger
    // TODO check original code for aok
    let needle = vec![0x00, 0xe0, 0xab, 0x45, 0x9a, 0x99, 0x99, 0x99, 0x99, 0x99, 0xf9, 0x3f];
    match h.rfind(&needle, 0..h.data().len()) {
        Some(pos) => r.debug.triggerpos = pos + needle.len(),
        None => return Err((r, "can't find triggerpos".to_string())),
    };

    // Find game settings
    let needle = vec![0x9d, 0xff, 0xff, 0xff];
    match h.rfind(&needle, 0..r.debug.triggerpos) {
        Some(pos) => r.debug.settingspos = pos - 64,
        None => return Err((r, "can't find game settings".to_string())),
    };

    // Find disabled techs
    r.debug.disabledtechspos = r.debug.settingspos - 5392;

    // Find victory pos
    r.debug.victorypos = r.debug.disabledtechspos - 12544 - 44;

    // Victory
    h.seek(r.debug.victorypos);
    h.mov(4);
    r.isconquest = h.get_bool(4);
    h.mov(4);
    r.relics2win = h.get_i32();
    h.mov(4);
    r.explored2win = h.get_i32();
    h.mov(4);
    r.anyorall = h.get_bool(4);
    r.victorymode = h.get_i32();
    r.score2win = h.get_i32();
    r.time2win = h.get_i32();

    // Find scenario pos
    let needle = match r.vercode.as_deref() {
        Some("aok") => vec![0x9a, 0x99, 0x99, 0x3f], // float 1.20
        _ => vec![0xf6, 0x28, 0x9c, 0x3f],           // float 1.22
    };
    match h.rfind(&needle, 0..r.debug.victorypos) {
        Some(pos) => r.debug.scenariopos = pos - 4,
        None => return Err((r, "can't find scenario pos".to_string())),
    };

    // Scenario filename & instructions
    h.seek(r.debug.scenariopos + 4433);
    r.scenariofilename = h.extract_str_l16();
    h.mov(4 * 5);
    if r.vercode.as_deref() != Some("aok") {
        h.mov(4);
    }
    r.instructions = h.extract_str_l16();

    // Skip trigger
    h.seek(r.debug.triggerpos);
    h.mov(1);
    let num_triggers = h.get_i32().unwrap();
    for i in 0..num_triggers {
        h.mov(18);
        let description_len = h.get_i32().unwrap();
        if description_len > 0 {
            h.mov(description_len as isize);
        }
        let name_len = h.get_i32().unwrap();
        if name_len > 0 {
            h.mov(name_len as isize);
        }
        let num_effects = h.get_i32().unwrap();
        for _ in 0..num_effects {
            h.mov(24);
            let mut num_selected_objs = h.get_i32().unwrap();
            if num_selected_objs == -1 {
                num_selected_objs = 0;
            }
            h.mov(72);
            let text_len = h.get_i32().unwrap();
            if text_len > 0 {
                h.mov(text_len as isize);
            }
            let sound_filename_len = h.get_i32().unwrap();
            if sound_filename_len > 0 {
                h.mov(sound_filename_len as isize);
            }
            h.mov(4 * num_selected_objs as isize);
        }
        h.mov(4 * num_effects as isize);
        let num_conditions = h.get_i32().unwrap();
        let condition_size = 72;
        h.mov((num_conditions * condition_size + num_conditions * 4) as isize);
    }
    if num_triggers > 0 {
        h.mov(4 * num_triggers as isize);
    }
    r.debug.lobbypos = h.tell();

    // Game settings
    h.seek(r.debug.initpos);
    h.mov(2 + r.totalplayers.unwrap() as isize + 36 + 4 + 1);
    h.extract_str_l16();
    let trail_types: [u8; 6] = h.current()[..6].try_into().unwrap();

    h.seek(r.debug.settingspos);
    h.mov(64 + 4 + 8);
    r.mapid = h.get_i32();
    r.difficultyid = h.get_i32();
    r.lockteams = h.get_bool(4);
    let mut init_search_needles = Vec::new();
    for i in 0..9 {
        r.players[i].index = h.get_i32();
        r.players[i].playertype = h.get_i32();
        if r.players[i].playertype.unwrap() == 4 {
            r.include_ai = Some(true);
        }
        let namelen = h.peek_i32().unwrap();
        let mut init_search_needle = Vec::new();
        init_search_needle.extend_from_slice((namelen as i16 + 1).to_le_bytes().as_ref());
        init_search_needle.extend_from_slice(&h.current()[4..4 + namelen as usize]);
        init_search_needle.push(b'\0');
        init_search_needle.extend_from_slice(trail_types.as_ref());
        init_search_needles.push(init_search_needle);
        r.players[i].name = h.extract_str_l32();
    }

    // Find data pos in init
    h.seek(r.debug.initpos + 2 + r.totalplayers.unwrap() as usize + 36 + 4 + 1);
    let mut easy_skip_start = h.tell() + 35100 + r.mapx.unwrap() as usize * r.mapy.unwrap() as usize;
    let search_end_pos = if r.debug.scenariopos != 0 {
        r.debug.scenariopos
    } else if r.debug.victorypos != 0 {
        r.debug.victorypos
    } else if r.debug.disabledtechspos != 0 {
        r.debug.disabledtechspos
    } else if r.debug.settingspos != 0 {
        r.debug.settingspos
    } else {
        h.data().len()
    } - r.totalplayers.unwrap() as usize * 1817;

    for i in 1..9 {
        if !r.players[i].isvalid()
            || r.players[i].index.is_none()
            || r.players[i]
                .index
                .is_some_and(|idx| idx < 0 || idx > 8 || r.debug.playerinitpos_by_idx[idx as usize].is_some())
        {
            continue;
        }
        let pos = h.find(init_search_needles.get(i).unwrap().clone(), easy_skip_start..search_end_pos);
        if pos.is_some() {
            r.debug.playerinitpos_by_idx[r.players[i].index.unwrap() as usize] = pos;
            h.seek(pos.unwrap());
            easy_skip_start = h.tell();
        }
    }

    // Lobby
    h.seek(r.debug.lobbypos);
    if r.versave.is_some_and(|v| v >= 13.3399) {
        h.mov(5);
    }
    if r.versave.is_some_and(|v| v >= 20.0599) {
        h.mov(9);
    }
    if r.versave.is_some_and(|v| v >= 26.1599) {
        h.mov(5);
    }
    if r.versave.is_some_and(|v| v >= 36.9999) {
        h.mov(8);
    }
    for i in 1..9 {
        r.players[i].teamid = h.get_u8();
    }
    if r.versave.is_some_and(|v| v < 12.2999) {
        h.mov(1);
    }
    r.revealmap = h.get_i32();
    h.mov(4); // fog of war
    r.mapsize = h.get_i32();
    r.poplimit = h.get_i32();
    if r.poplimit.unwrap() < 40 {
        r.poplimit = r.poplimit.map(|pop| pop * 25);
    }
    r.gametype = h.get_u8();
    r.lockdiplomacy = h.get_bool(1);
    if !r.vercode.as_ref().is_some_and(|v| v == "aok") {
        let totalchats = h.get_i32().unwrap();
        for _ in 0..totalchats {
            r.chat.push(Chat { 
                time: None,
                player: None,
                message: h.extract_str_l32(),
             });
        }
    }

    // Init data
    for i in 0..9 {
        let pos_by_idx = r.debug.playerinitpos_by_idx[r.players[i].index.unwrap() as usize];
        if r.players[i].isvalid() && pos_by_idx.is_some() {
            h.seek(pos_by_idx.unwrap());
            let mainop_name = h.extract_str_l16();
            if mainop_name == r.players[i].name {
                r.players[i].ismainop = Some(true);
            } else {
                r.players[i].ismainop = Some(false);
            }
            h.mov(762);
            if r.versave.is_some_and(|v| v >= 11.7599) {
                h.mov(36);
            }
            // TODO: see mgxparser
            h.mov(1);
            r.players[i].initx = h.get_f32();
            r.players[i].inity = h.get_f32();

            if r.vercode.as_deref() != Some("aok") {
                let num_savedviews = h.get_i32().unwrap();
                if num_savedviews > 0 {
                    h.mov(num_savedviews as isize * 8);
                }
            }

            h.mov(5);
            r.players[i].civid = h.get_u8();
            h.mov(3);
            r.players[i].colorid = h.get_u8();
        }
    }

    // for i in 0..128 {
    //     // print hex
    //     if i % 16 == 0 {
    //         print!("\n{:04x}: ", i);
    //     }
    //     print!("{:02x?} ", h.current()[i]);
    // }

    // Body
    let sync_checksum_interval = 500;
    if b.peek_u32().unwrap() != sync_checksum_interval {
        b.mov(4); // TODO verlog
    }
    b.mov(4); // Sync checksum interval
    r.ismultiplayer = b.get_bool(4);
    b.mov(16);
    if b.remain() >= 4 && b.peek_u32().unwrap() == 0 {
        b.mov(4);
    } else if b.remain() >= 8 && b.peek_u32().unwrap() != 2 {
        b.mov(8);
    }

    const OP_COMMAND: i32 = 0x01;
    const OP_SYNC: i32 = 0x02;
    const OP_VIEWLOCK: i32 = 0x03;
    const OP_CHAT: i32 = 0x04;
    while b.remain() >= 4 {
        let op_type = b.get_i32().unwrap();
        match op_type {
            OP_COMMAND => {
                let cmdlen = b.get_u32().unwrap();
                let nextpos = if b.remain() < cmdlen as usize { b.data().len() } else { b.tell() + cmdlen as usize };
                const COMMAND_RESIGN: u8 = 0x0b;
                const COMMAND_RESEARCH: u8 = 0x65;
                const COMMAND_TRAIN: u8 = 0x77;
                const COMMAND_TRAIN_SINGLE: u8 = 0x64;
                const COMMAND_BUILD: u8 = 0x66;
                const COMMAND_TRIBUTE: u8 = 0x6c;
                const COMMAND_POSTGAME: u8 = 0xff;
                const COMMAND_MOVE: u8 = 0x03;
                const COMMAND_SAVE: u8 = 0x1b;

                let cmd = b.get_u8().unwrap();
                match cmd {
                    COMMAND_RESIGN => {
                        // Handle resign command
                    }
                    COMMAND_RESEARCH => {
                        // Handle research command
                    }
                    COMMAND_TRAIN => {
                        // Handle train command
                    }
                    COMMAND_TRAIN_SINGLE => {
                        // Handle train single command
                    }
                    COMMAND_BUILD => {
                        // Handle build command
                    }
                    COMMAND_TRIBUTE => {
                        // Handle tribute command
                    }
                    COMMAND_POSTGAME => {
                        // Handle postgame command
                    }
                    COMMAND_MOVE => {
                        // Handle move command
                    }
                    COMMAND_SAVE => {
                        // Handle save command
                    }
                    _ => {
                        // Handle unknown command
                    }
                }

                b.seek(nextpos);
            }
            OP_SYNC => {
                r.duration += b.get_u32().unwrap();
                let sync_data = b.get_u32().unwrap();
                b.mov(if sync_data == 0 { 28 } else { 0 });
                b.mov(12);
            }
            OP_VIEWLOCK => {
                println!("OP_VIEWLOCK");
                b.mov(12);
            }
            OP_CHAT => {
                if b.peek_i32().unwrap() != -1 {
                    continue;
                } else {
                    b.mov(4);
                }
                let msg = b.extract_str_l32();
                if msg.as_ref().is_some_and(|s| s.len() > 0) {
                    if msg.as_ref().unwrap().starts_with(b"@#")
                        && *msg.as_ref().unwrap().get(3).unwrap() == b'-'
                        && *msg.as_ref().unwrap().get(3).unwrap() == b'-'
                    {
                        continue;
                    }
                    r.chat.push(Chat { time: Some(r.duration), player: None, message: msg });
                }
            }
            _ => {}
        }
    }
    // draw_map(&h, &r, "test.png");
    Ok((r, "".to_string()))
}
