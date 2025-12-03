use crate::body_parser::parse_body;
use crate::cursor::StreamCursor;
use crate::guess_winner::guess;
use crate::guid::calc_guid;
use crate::record::*;
use crate::val;
use anyhow::{bail, Ok, Result};
use chksum_hash_md5 as md5;
use flate2::read::ZlibDecoder;
use flate2::Decompress;
use std::io::Read;

/// Recorded game parser. Used to parse recorded game file
pub struct Parser<T: AsRef<[u8]>> {
    pub header: StreamCursor<Vec<u8>>,
    pub body: StreamCursor<T>,
    pub md5: String,
}

impl<T: AsRef<[u8]>> Parser<T> {
    /// Input buffer will be consumed
    pub fn new(src: T) -> Result<Self> {
        let md5 = md5::hash(&src).to_hex_lowercase();

        let b = src.as_ref();
        // Sometimes header length is missing(always 0x00), calculate actual header length with decompressed length is more reliable
        let mut rawheader_end = u32::from_le_bytes(b[0..4].try_into()?);
        let nextpos = u32::from_le_bytes(b[4..8].try_into()?);
        let rawheader_begin = if nextpos < b.len() as u32 { 8 } else { 4 };

        let mut header_buffer = Vec::new();
        let mut decoder = ZlibDecoder::new_with_decompress(&b[rawheader_begin as usize..], Decompress::new(false));
        decoder.read_to_end(&mut header_buffer)?;
        let compressed_size = decoder.total_in(); // Get compressed size

        if rawheader_end < compressed_size as u32 + rawheader_begin {
            rawheader_end = compressed_size as u32 + rawheader_begin;
        }

        #[cfg(debug_assertions)]
        if rawheader_begin == 8 && nextpos > 0 {
            let mut chapter_buffer = Vec::new();
            match ZlibDecoder::new_with_decompress(&b[nextpos as usize + 8..], Decompress::new(false))
                .read_to_end(&mut chapter_buffer)
            {
                Err(_) => bail!("Failed to decompress chapter header"),
                _ => {
                    // println!("Decompressed chapter header size: {}", chapter_buffer.len());
                    // write test body to file
                    // std::fs::write("testbody.bin", &test_buffer).unwrap();
                }
            }
        }

        let header = StreamCursor::new(header_buffer, 0); // header cursor
        let body = StreamCursor::new(src, rawheader_end as usize); // body cursor

        Ok(Parser { header, body, md5 })
    }

    pub fn dump_header(&self, filename: &str) -> Result<()> {
        std::fs::write(filename, self.header.data())?;
        Ok(())
    }

    pub fn dump_body(&self, filename: &str) -> Result<()> {
        std::fs::write(filename, self.body.data())?;
        Ok(())
    }

    /// Try to extract info from the recorded game.   
    /// Parsing may not be complete. Check `None` for fields when using `Record`.
    pub fn parse_to(self: &mut Self, r: &mut Record) -> Result<&mut Self> {
        r.md5 = Some(self.md5.clone());

        let h = &mut self.header;

        let verraw: [u8; 7] = h.current()[0..7].try_into()?;
        r.verraw = Some(String::from_utf8_lossy(&verraw).to_string());
        h.mov(8);
        r.versave = h.get_f32();
        if -1.0 == val!(r.versave) {
            r.versave2 = h.get_u32();
        }
        let verlog_check = self.body.peek_u32();
        if verlog_check == Some(500) {
            r.ver = Some(Version::AoK)
        } else {
            r.verlog = verlog_check;
        }
        match &verraw {
            b"TRL 9.3" => {
                if r.ver == Some(Version::AoK) {
                    r.ver = Some(Version::AoKTrial);
                } else {
                    r.ver = Some(Version::AoCTrial);
                }
            }
            b"VER 9.3" => r.ver = Some(Version::AoK),
            b"VER 9.4" => {
                if val!(r.verlog) == 0 || val!(r.verlog) == 3 {
                    r.ver = Some(Version::AoC10a);
                } else if val!(r.verlog) == 5 || val!(r.versave) >= 12.9699 {
                    r.ver = Some(Version::DE);
                } else if val!(r.versave) > 11.7601 {
                    r.ver = Some(Version::HD);
                } else if val!(r.verlog) == 4 {
                    r.ver = Some(Version::AoC10c);
                } else {
                    r.ver = Some(Version::AoC);
                }
            }
            b"VER 9.5" => r.ver = Some(Version::AoFE21),
            b"VER 9.8" => r.ver = Some(Version::UP12),
            b"VER 9.9" => r.ver = Some(Version::UP13),
            b"VER 9.A" => r.ver = Some(Version::UP14RC1),
            b"VER 9.B" => r.ver = Some(Version::UP14RC2),
            b"VER 9.C" | b"VER 9.D" => r.ver = Some(Version::UP14),
            b"VER 9.E" | b"VER 9.F" => r.ver = Some(Version::UP15),
            b"MCP 9.F" => r.ver = Some(Version::MCP),
            _ => r.ver = Some(Version::Unknown),
        }

        // This parser don't support de/hd versions
        if val!(r.versave) >= 11.7601 || val!(r.versave) < 0.0 {
            bail!("DE/HD or higher versions are not supported");
        }

        // https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/HeaderAnalyzer.php#L305
        r.debug.aipos = h.tell();
        r.include_ai = h.get_bool(4);
        if val!(r.include_ai) {
            h.mov(2);
            let num_ai_strings = val!(h.get_u16());
            h.mov(4);
            for _ in 0..num_ai_strings {
                let str_len = val!(h.get_u32());
                h.mov(str_len as isize);
            }
            h.mov(6);

            let action_size = 24;
            let rule_size = 16 + 16 * action_size;
            for _ in 0..8 {
                h.mov(10);
                let num_rules = val!(h.get_u16());
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
        r.speed_raw = h.get_u32();
        h.mov(29);
        r.recorder = h.get_u16();
        r.totalplayers = h.get_u8();
        if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
            r.instantbuild = h.get_bool(1);
            r.enablecheats = h.get_bool(1);
        }
        h.mov(2 + 58);

        // https://github.com/goto-bus-stop/recanalyst/blob/master/src/Analyzers/MapDataAnalyzer.php#L7
        r.mapx = h.get_i32();
        r.mapy = h.get_i32();
        if val!(r.mapx) < 0 || val!(r.mapy) < 0 {
            bail!("Map size is negative");
        } else if val!(r.mapx) > 10000 || val!(r.mapy) > 10000 {
            bail!("Map size is too large");
        } else if val!(r.mapx) != val!(r.mapy) {
            bail!("Map is not square");
        }

        let num_mapzones = val!(h.get_i32());
        let map_bits: isize = (val!(r.mapx) * val!(r.mapy)) as isize;
        for _ in 0..num_mapzones as usize {
            h.mov(1275 + map_bits);
            let num_floats = val!(h.get_i32());
            h.mov(num_floats as isize * 4 + 4);
        }
        r.nofog = h.get_bool(1);
        h.mov(1);

        r.debug.mappos = Some(h.tell());

        let maptile_type: isize = if val!(h.peek_u8()) == 255 { 4 } else { 2 };
        h.mov(map_bits * maptile_type);
        let num_data = val!(h.get_i32()) as isize;
        h.mov(4 + num_data * 4);
        for _ in 0..num_data {
            let num_obstructions = val!(h.get_i32()) as isize;
            h.mov(num_obstructions * 8);
        }
        let visibility_mapsize0 = val!(h.get_i32()) as isize;
        let visibility_mapsize1 = val!(h.get_i32()) as isize;
        h.mov(visibility_mapsize0 * visibility_mapsize1 * 4);
        r.restoretime = h.get_u32();
        let num_particles = val!(h.get_u32());
        h.mov(27 * num_particles as isize + 4);

        r.debug.initpos = h.tell();
        if r.ver == Some(Version::AoKTrial) {
            r.debug.initpos += 4;
        }

        // Find trigger
        let needle = vec![0x9a, 0x99, 0x99, 0x99, 0x99, 0x99, 0xf9, 0x3f];
        match h.rfind(&needle, 0..h.data().len()) {
            Some(pos) => r.debug.triggerpos = pos + needle.len(),
            None => bail!("can't find triggerpos"),
        };

        // Skip trigger
        h.seek(r.debug.triggerpos);
        h.mov(1);
        let num_triggers = val!(h.get_i32());
        for _ in 0..num_triggers {
            h.mov(4 + (2 * 1) + (3 * 4));
            let description_len = val!(h.get_i32());
            if description_len > 0 {
                h.mov(description_len as isize);
            }
            let name_len = val!(h.get_i32());
            if name_len > 0 {
                h.mov(name_len as isize);
            }
            let num_effects = val!(h.get_i32());
            for _ in 0..num_effects {
                h.mov(24);
                let mut num_selected_objs = val!(h.get_i32());
                if num_selected_objs == -1 {
                    num_selected_objs = 0;
                }
                h.mov(72);
                let text_len = val!(h.get_i32());
                if text_len > 0 {
                    h.mov(text_len as isize);
                }
                let sound_filename_len = val!(h.get_i32());
                if sound_filename_len > 0 {
                    h.mov(sound_filename_len as isize);
                }
                h.mov(4 * num_selected_objs as isize);
            }
            h.mov(4 * num_effects as isize);
            let num_conditions = val!(h.get_i32());
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
        r.revealmap_raw = h.get_i32();
        h.mov(4); // fog of war
        r.mapsize_raw = h.get_u32();
        r.poplimit = h.get_u32();
        if val!(r.poplimit) <= 40 {
            r.poplimit = r.poplimit.map(|pop| pop * 25);
        }
        if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
            r.gametype_raw = h.get_u8();
            r.lockdiplomacy = h.get_bool(1);

            let totalchats = val!(h.get_i32());
            for _ in 0..totalchats {
                if h.peek_i32() == Some(0) {
                    h.mov(4);
                    continue;
                }
                r.chat.push(Chat { time: None, player: None, content_raw: h.extract_str_l32(), content: None });
            }
        }

        // Find game settings
        let needle = vec![0x9d, 0xff, 0xff, 0xff];
        match h.rfind(&needle, 0..r.debug.triggerpos) {
            Some(pos) => r.debug.settingspos = pos,
            None => bail!("can't find settingspos"),
        };

        // Locate disabled techs
        r.debug.disabledtechspos = r.debug.settingspos - 5456;

        // Locate victory pos
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
        r.victorytype_raw = h.get_i32();
        r.score2win = h.get_i32();
        r.time2win_raw = h.get_i32();

        // Find scenario pos
        let needle = match &r.ver {
            Some(Version::AoK) | Some(Version::AoKTrial) => vec![0x9a, 0x99, 0x99, 0x3f], // float 1.20
            _ => vec![0xf6, 0x28, 0x9c, 0x3f],                                            // float 1.22
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
        r.scenariofilename_raw = h.extract_str_l16();
        h.mov(4 * 5);
        if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
            h.mov(4);
        }
        r.instructions_raw = h.extract_str_l16();

        // Game settings
        h.seek(r.debug.initpos);
        h.mov(2 + val!(r.totalplayers) as isize + 36 + 4 + 1);
        h.extract_str_l16();
        let trail_types: [u8; 6] = h.current()[..6].try_into()?;

        h.seek(r.debug.settingspos);

        h.mov(4 + 8);
        if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
            r.mapid = h.get_u32();
        }
        r.difficulty_raw = h.get_i32();
        r.lockteams = h.get_bool(4);
        let mut init_search_needles = Vec::new();
        for i in 0..9 {
            r.players[i].index = h.get_i32();
            r.players[i].playertype = h.get_i32();
            let namelen = val!(h.peek_i32());
            let mut init_search_needle = Vec::new();
            init_search_needle.extend_from_slice((namelen as i16 + 1).to_le_bytes().as_ref());
            init_search_needle.extend_from_slice(&h.current()[4..4 + namelen as usize]);
            init_search_needle.push(b'\0');
            init_search_needle.extend_from_slice(trail_types.as_ref());
            init_search_needles.push(init_search_needle);
            r.players[i].name_raw = h.extract_str_l32();
        }

        // Find data pos in init
        h.seek(r.debug.initpos + 2 + val!(r.totalplayers) as usize + 36 + 4 + 1);
        let mut easy_skip_start = h.tell() + 35100 + val!(r.mapx) as usize * val!(r.mapy) as usize;
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
        } - val!(r.totalplayers) as usize * 1817;

        for i in 1..9 {
            if !r.players[i].isvalid()
                || r.players[i].index.is_none()
                || r.players[i]
                    .index
                    .is_some_and(|idx| idx < 0 || idx > 8 || r.debug.playerinitpos_by_idx[idx as usize].is_some())
            {
                continue;
            }

            if let Some(needle) = init_search_needles.get(i).cloned() {
                let pos = h.find(needle, easy_skip_start..search_end_pos);
                if pos.is_some() {
                    r.debug.playerinitpos_by_idx[val!(r.players[i].index) as usize] = pos;
                    h.seek(val!(pos));
                    easy_skip_start = h.tell();
                } else {
                    // print a hint only in debug mode
                    #[cfg(debug_assertions)]
                    {
                        println!("Player {} not found, needle: {:02x?}", i, init_search_needles.get(i).cloned());
                    }
                }
            }
        }

        // Analyze diplomacy
        let mut playerpos = r.debug.playerinitpos_by_idx.clone();
        let totalplayers = val!(r.totalplayers) as usize;
        for i in 1..9 {
            if r.players[i].index.is_none() {
                continue;
            }
            let idx = val!(r.players[i].index) as usize;
            if playerpos[idx].is_some() && r.players[i].isvalid() {
                let mut team_members = vec![idx as i32];
                let pos_my_diplomacy = val!(playerpos[idx]) - (5 + 36);
                let pos_diplomacy = pos_my_diplomacy - totalplayers; // first one is GAIA
                for j in (idx + 1)..totalplayers {
                    h.seek(pos_diplomacy + j);
                    let other_to_me = val!(h.get_u8()) as i32;
                    h.seek(pos_my_diplomacy + j * 4);
                    let me_to_other = val!(h.get_i32());
                    // println!("Player slot#{} to index#{}: {} -> {}", i, j, other_to_me, me_to_other);
                    if other_to_me == 0 && me_to_other == 2 {
                        team_members.push(j as i32);
                        playerpos[j] = None; // This player don't need to be checked again
                    }
                }
                r.teams.push(team_members);
                playerpos[idx] = None; // This player don't need to be checked again
            }
        }
        // create a var team_count, contains the number of players in each team order by asc
        let mut team_count: Vec<usize> = r.teams.iter().map(|t| t.len()).collect();
        team_count.sort();
        r.matchup = Some(team_count);

        // Init data
        for i in 0..9 {
            let pos_by_idx = r.debug.playerinitpos_by_idx[val!(r.players[i].index) as usize];
            // Which is put before '&&' makes a difference
            if r.players[i].isvalid() && pos_by_idx.is_some() {
                h.seek(val!(pos_by_idx));
                let mainop_name = h.extract_str_l16();
                if mainop_name == r.players[i].name_raw {
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
                r.players[i].initage_raw = h.get_f32();
                h.mov(4 * 4);
                r.players[i].initpop = h.get_f32();
                h.mov(4 * 25);
                r.players[i].initcivilian = h.get_f32();
                h.mov(4 * 2);
                r.players[i].initmilitary = h.get_f32();
                h.mov(756 - 41 * 4);
                if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
                    h.mov(36);
                }
                if r.ver == Some(Version::UP15) || r.ver == Some(Version::MCP) {
                    r.players[i].modversion = h.get_f32();
                    h.mov(4 * 6 + 4 * 7 + 4 * 28);
                }
                h.mov(1);
                r.players[i].initx = h.get_f32();
                r.players[i].inity = h.get_f32();

                if r.ver != Some(Version::AoK) && r.ver != Some(Version::AoKTrial) {
                    let num_savedviews = val!(h.get_i32());
                    if num_savedviews > 0 {
                        h.mov(num_savedviews as isize * 8);
                    }
                }

                h.mov(5);
                r.players[i].civ_raw = h.get_u8();
                h.mov(3);
                r.players[i].colorid = h.get_u8();
            }
        }

        // Body
        let b = &mut self.body;

        if r.ver == Some(Version::AoK) || r.ver == Some(Version::AoKTrial) {
            debug_assert!(val!(b.peek_i32()) == 500);
            b.mov(36);
        } else {
            b.mov(4);
            debug_assert!(val!(b.peek_i32()) == 500);
            b.mov(4); // interval
            r.ismultiplayer = b.get_bool(4);
            b.mov(16);
        }

        let mut next_chapter_pos = u32::from_le_bytes(b.src.as_ref()[4..8].try_into()?);
        if r.ver == Some(Version::AoK) || r.ver == Some(Version::AoKTrial) || next_chapter_pos == 0 {
            parse_body(b, r)?;
        } else {
            let mut start = b.pos_in_data + b.offset;
            let mut end = next_chapter_pos as usize;
            loop {
                let body_slice = b.src.as_ref()[start..end].to_vec();
                let mut slice_stream = StreamCursor::new(body_slice, 0);
                parse_body(&mut slice_stream, r)?;

                if end >= b.src.as_ref().len() {
                    break;
                }

                start = u32::from_le_bytes(b.src.as_ref()[end..end + 4].try_into()?) as usize;
                next_chapter_pos = u32::from_le_bytes(b.src.as_ref()[end + 4..end + 8].try_into()?);
                if next_chapter_pos == 0 {
                    end = b.src.as_ref().len();
                } else {
                    end = next_chapter_pos as usize;
                }
            }
        }

        r.guid = Some(calc_guid(r)?);
        guess(r)?;

        Ok(self)
    }
}
