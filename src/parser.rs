use crate::cursor::StreamCursor;
use crate::record::{Chat, Record, Version};
use anyhow::{bail, Ok, Result};
use flate2::read::ZlibDecoder;
use flate2::Decompress;
use std::io::Read;

pub struct Parser {
    pub header: StreamCursor,
    pub body: StreamCursor,
    /// (mapdata offset in header, mapx, mapy)
    pub mapattr: (Option<usize>, Option<i32>, Option<i32>),
}

impl Parser {
    pub fn new(b: Vec<u8>) -> Result<Self> {
        let rawheader_end = u32::from_le_bytes(b[0..4].try_into()?);
        let nextpos = u32::from_le_bytes(b[4..8].try_into()?);
        let rawheader_begin = if nextpos < b.len() as u32 { 8 } else { 4 };

        let mut header_buffer = Vec::new();
        let rawheader = &b[rawheader_begin as usize..];
        ZlibDecoder::new_with_decompress(rawheader, Decompress::new(false)).read_to_end(&mut header_buffer)?;

        let header = StreamCursor::new(header_buffer, 0); // header cursor
        let body = StreamCursor::new(b, rawheader_end as usize); // body cursor

        Ok(Parser { header, body, mapattr: (None, None, None) })
    }

    pub fn dump_header(&self, filename: &str) -> Result<()> {
        std::fs::write(filename, &self.header.data())?;
        Ok(())
    }

    pub fn dump_body(&self, filename: &str) -> Result<()> {
        std::fs::write(filename, &self.body.data())?;
        Ok(())
    }

    /// Try to extract info from the recorded game.   
    /// Parsing may not be complete. Check keys in the `Record` when using it.
    pub fn parse_to(self: &mut Self, r: &mut Record) -> Result<&mut Self> {
        let h = &mut self.header;

        r.verraw = h.current()[0..8].try_into()?;
        h.mov(8);
        r.versave = h.get_f32();
        if -1.0 == r.versave.unwrap() {
            r.versave2 = h.get_u32();
        }
        if r.ver != Some(Version::AoK) {
            r.verlog = self.body.peek_u32();
        }
        match &r.verraw {
            b"TRL 9.3\0" => {
                if r.ver == Some(Version::AoK) {
                    r.ver = Some(Version::AoKTrial);
                } else {
                    r.ver = Some(Version::AoCTrial);
                }
            }
            b"VER 9.3\0" => r.ver = Some(Version::AoK),
            b"VER 9.4\0" => {
                if r.verlog.unwrap() == 0 || r.verlog.unwrap() == 3 {
                    r.ver = Some(Version::AoC10a);
                } else if r.verlog.unwrap() == 5 || r.versave.unwrap() >= 12.9699 {
                    r.ver = Some(Version::DE);
                } else if r.versave.unwrap() > 11.7601 {
                    r.ver = Some(Version::HD);
                } else if r.verlog.unwrap() == 4 {
                    r.ver = Some(Version::AoC10c);
                } else {
                    r.ver = Some(Version::AoC);
                }
            }
            b"VER 9.5\0" => r.ver = Some(Version::AoFE21),
            b"VER 9.8\0" => r.ver = Some(Version::UP12),
            b"VER 9.9\0" => r.ver = Some(Version::UP13),
            b"VER 9.A\0" => r.ver = Some(Version::UP14RC1),
            b"VER 9.B\0" => r.ver = Some(Version::UP14RC2),
            b"VER 9.C\0" | b"VER 9.D\0" => r.ver = Some(Version::UP14),
            b"VER 9.E\0" | b"VER 9.F\0" => r.ver = Some(Version::UP15),
            b"MCP 9.F\0" => r.ver = Some(Version::MCP),
            _ => r.ver = Some(Version::Unknown),
        }

        // This parser don't support de/hd versions
        if r.versave.unwrap() >= 11.7601 || r.versave.unwrap() < 0.0 {
            bail!("DE/HD or higher versions are not supported");
        }

        // https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/HeaderAnalyzer.php#L305
        r.debug.aipos = h.tell();
        r.include_ai = h.get_bool(4);
        if r.include_ai.unwrap() {
            h.mov(2);
            let num_ai_strings = h.get_u16().unwrap();
            h.mov(4);
            for _ in 0..num_ai_strings {
                let str_len = h.get_u32().unwrap();
                h.mov(str_len as isize);
            }
            h.mov(6);

            let action_size = 24;
            let rule_size = 16 + 16 * action_size;
            for _ in 0..8 {
                h.mov(10);
                let num_rules = h.get_u16().unwrap();
                h.mov(4);
                for _ in 0..num_rules {
                    h.mov(rule_size as isize);
                }
            }
            h.mov(104 + 320 + 1024);
            h.mov(4096);
        }

        // https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/HeaderAnalyzer.php#L68
        h.mov(12);
        r.speed = h.get_u32();
        h.mov(29);
        r.recorder = h.get_u16();
        r.totalplayers = h.get_u8();
        if r.ver != Some(Version::AoK) {
            r.instantbuild = h.get_bool(1);
            r.enablecheats = h.get_bool(1);
        }
        h.mov(2 + 58);

        // https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/MapDataAnalyzer.php#L7
        r.mapx = h.get_i32();
        r.mapy = h.get_i32();
        if r.mapx.unwrap() < 0 || r.mapy.unwrap() < 0 {
            bail!("Map size is negative");
        } else if r.mapx.unwrap() > 10000 || r.mapy.unwrap() > 10000 {
            bail!("Map size is too large");
        } else if r.mapx.unwrap() != r.mapy.unwrap() {
            bail!("Map is not square");
        }
        self.mapattr.1 = r.mapx;
        self.mapattr.2 = r.mapy;

        let num_mapzones = h.get_i32().unwrap();
        let map_bits: isize = (r.mapx.unwrap() * r.mapy.unwrap()) as isize;
        for _ in 0..num_mapzones as usize {
            h.mov(1275 + map_bits);
            let num_floats = h.get_i32().unwrap();
            h.mov(num_floats as isize * 4 + 4);
        }
        r.nofog = h.get_bool(1);
        h.mov(1);

        self.mapattr.0 = Some(h.tell());

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
        let needle = vec![0x9a, 0x99, 0x99, 0x99, 0x99, 0x99, 0xf9, 0x3f];
        match h.rfind(&needle, 0..h.data().len()) {
            Some(pos) => r.debug.triggerpos = pos + needle.len(),
            None => bail!("can't find triggerpos"),
        };

        // Skip trigger
        h.seek(r.debug.triggerpos);
        h.mov(1);
        let num_triggers = h.get_i32().unwrap();
        for _ in 0..num_triggers {
            h.mov(4 + (2 * 1) + (3 * 4));
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

        // Lobby
        for i in 1..9 {
            r.players[i].teamid = h.get_u8();
        }
        h.mov(1);
        r.revealmap = h.get_i32();
        h.mov(4); // fog of war
        r.mapsize = h.get_u32();
        r.poplimit = h.get_u32();
        if r.poplimit.unwrap() < 40 {
            r.poplimit = r.poplimit.map(|pop| pop * 25);
        }
        if r.ver != Some(Version::AoK) {
            r.gametype = h.get_u8();
            r.lockdiplomacy = h.get_bool(1);

            let totalchats = h.get_i32().unwrap();
            for _ in 0..totalchats {
                r.chat.push(Chat { time: None, player: None, message: h.extract_str_l32() });
            }
        }

        // Find game settings
        let needle = vec![0x9d, 0xff, 0xff, 0xff];
        match h.rfind(&needle, 0..r.debug.triggerpos) {
            Some(pos) => r.debug.settingspos = pos,
            None => bail!("can't find settingspos"),
        };

        // Find disabled techs
        r.debug.disabledtechspos = r.debug.settingspos - 5456;

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
        let needle = match r.ver {
            Some(Version::AoK) => vec![0x9a, 0x99, 0x99, 0x3f], // float 1.20
            _ => vec![0xf6, 0x28, 0x9c, 0x3f],                  // float 1.22
        };
        match h.rfind(&needle, 0..r.debug.victorypos) {
            Some(pos) => {
                r.debug.scenariopos = pos - 4;
            }
            None => bail!("can't find scenariopos"),
        };
        h.seek(r.debug.scenariopos);
        h.mov(4);
        r.verscenario = h.get_f32();
        h.mov(16 * 256 + 16 * 4 + 16 * 16 + 5 + 4);
        r.scenariofilename = h.extract_str_l16();
        h.mov(4 * 5);
        if r.ver != Some(Version::AoK) {
            h.mov(4);
        }
        r.instructions = h.extract_str_l16();

        // Game settings
        h.seek(r.debug.initpos);
        h.mov(2 + r.totalplayers.unwrap() as isize + 36 + 4 + 1);
        h.extract_str_l16();
        let trail_types: [u8; 6] = h.current()[..6].try_into()?;

        h.seek(r.debug.settingspos);

        h.mov(4 + 8);
        if r.ver != Some(Version::AoK) {
            r.mapid = h.get_u32();
        }
        r.difficultyid = h.get_i32();
        r.lockteams = h.get_bool(4);
        let mut init_search_needles = Vec::new();
        for i in 0..9 {
            r.players[i].index = h.get_i32();
            r.players[i].playertype = h.get_i32();
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
                h.mov(6);
                r.players[i].initfood = h.get_f32();
                r.players[i].initwood = h.get_f32();
                r.players[i].initstone = h.get_f32();
                r.players[i].initgold = h.get_f32();
                h.mov(8);
                r.players[i].initage = h.get_f32();
                h.mov(4 * 4);
                r.players[i].initpop = h.get_f32();
                h.mov(4 * 25);
                r.players[i].initcivilian = h.get_f32();
                h.mov(4 * 2);
                r.players[i].initmilitary = h.get_f32();
                h.mov(756 - 41 * 4);
                if r.ver != Some(Version::AoK) {
                    h.mov(36);
                }
                if r.ver == Some(Version::UP15) || r.ver == Some(Version::MCP) {
                    r.players[i].modversion = h.get_f32();
                    h.mov(4 * 6 + 4 * 7 + 4 * 28);
                }
                h.mov(1);
                r.players[i].initx = h.get_f32();
                r.players[i].inity = h.get_f32();

                if r.ver != Some(Version::AoK) {
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

        // Body
        let b = &mut self.body;

        let sync_checksum_interval = 500;
        if b.peek_u32().unwrap() != sync_checksum_interval {
            b.mov(4);
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
                    let nextpos =
                        if b.remain() < cmdlen as usize { b.data().len() } else { b.tell() + cmdlen as usize };
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
                            let slot = b.get_i8().unwrap();
                            if slot >= 0 && slot < 9 && r.players[slot as usize].isvalid() {
                                r.players[slot as usize].resigned = Some(r.duration);
                                r.players[slot as usize].disconnected = b.get_bool(4);
                            }
                        }
                        COMMAND_RESEARCH => {
                            b.mov(7);
                            let slot = b.get_i8().unwrap();
                            if slot < 0 || slot > 8 || !r.players[slot as usize].isvalid() {
                                break;
                            }
                            b.mov(1);
                            let techid = b.get_i16().unwrap();
                            match techid {
                                101 => r.players[slot as usize].feudaltime = Some(r.duration + 130000),
                                102 => {
                                    r.players[slot as usize].castletime = Some(
                                        r.duration
                                            + match r.players[slot as usize].civid {
                                                Some(8) => 160000 / 1.10 as u32,
                                                _ => 160000,
                                            },
                                    )
                                }
                                103 => {
                                    r.players[slot as usize].imperialtime = Some(
                                        r.duration
                                            + match r.players[slot as usize].civid {
                                                Some(8) => 190000 / 1.10 as u32,
                                                _ => 190000,
                                            },
                                    )
                                }
                                _ => {}
                            }
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
                            const EARLYMOVE_THRESHOLD: usize = 5;
                            if r.debug.earlymovecount < EARLYMOVE_THRESHOLD {
                                r.debug.earlymovecmd.push(b.current()[0]);
                                r.debug.earlymovetime.push(r.duration);
                                r.debug.earlymovecount += 1;
                            }
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
                    b.mov(12);
                }
                OP_CHAT => {
                    if b.peek_i32().unwrap() != -1 {
                        continue;
                    } else {
                        b.mov(4);
                    }
                    let msg = b.extract_str_l32();
                    if let Some(message) = msg.as_ref() {
                        if message.len() >= 5
                            && message.starts_with(b"@#")
                            && message.ends_with(b"--")
                            && message[3] == b'-'
                            && message[4] == b'-'
                            || message.len() == 0
                        {
                            continue;
                        }

                        r.chat.push(Chat { time: Some(r.duration), player: None, message: msg });
                        // TODO: player}
                    }
                }
                _ => {}
            }
        }
        Ok(self)
    }
}
