use crate::cursor::StreamCursor;
use crate::record::Chat;
use crate::record::Record;
use crate::record::Version;
use crate::val;
use anyhow::{bail, Ok, Result};

pub fn parse_body<T: AsRef<[u8]>>(b: &mut StreamCursor<T>, r: &mut Record) -> Result<()> {
    const OP_COMMAND: i32 = 0x01;
    const OP_SYNC: i32 = 0x02;
    const OP_VIEWLOCK: i32 = 0x03;
    const OP_CHAT: i32 = 0x04;

    const COMMAND_RESIGN: u8 = 0x0b;
    const COMMAND_RESEARCH: u8 = 0x65;
    const COMMAND_TRAIN: u8 = 0x77;
    const COMMAND_TRAIN_SINGLE: u8 = 0x64;
    const COMMAND_BUILD: u8 = 0x66;
    const COMMAND_TRIBUTE: u8 = 0x6c;
    const COMMAND_POSTGAME: u8 = 0xff;
    const COMMAND_MOVE: u8 = 0x03;
    const COMMAND_SAVE: u8 = 0x1b;
    const COMMAND_CHAPTER: u8 = 0x20;

    debug_assert!({
        if b.remain() >= 4 {
            val!(b.peek_i32()) == OP_SYNC
        } else {
            true
        }
    });

    while b.remain() >= 8 {
        let op_type = val!(b.get_i32());
        match op_type {
            OP_COMMAND => {
                let cmdlen = val!(b.get_u32()) + 4;
                let nextpos = if b.remain() < cmdlen as usize { b.data().len() } else { b.tell() + cmdlen as usize };

                let cmd = val!(b.get_u8());
                match cmd {
                    COMMAND_RESIGN => {
                        let slot = val!(b.get_i8());
                        b.mov(1);
                        if slot >= 0 && slot < 9 && r.players[slot as usize].isvalid() {
                            r.players[slot as usize].resigned = Some(r.duration);
                            r.players[slot as usize].disconnected = b.get_bool(4);
                        }
                    }
                    COMMAND_RESEARCH => {
                        b.mov(7);
                        let slot = val!(b.get_i8());
                        if slot < 0 || slot > 8 || !r.players[slot as usize].isvalid() {
                            ()
                        }
                        b.mov(1);
                        let techid = val!(b.get_i16());
                        match techid {
                            101 => r.players[slot as usize].feudaltime = Some(r.duration + 130000),
                            102 => {
                                if let Some(civ_raw) = r.players[slot as usize].civ_raw {
                                    r.players[slot as usize].castletime = Some(
                                        r.duration
                                            + match civ_raw {
                                                8 => 160000 / 1.10 as u32,
                                                _ => 160000,
                                            },
                                    )
                                } else {
                                    r.players[slot as usize].castletime = Some(r.duration + 160000)
                                }
                            }
                            103 => {
                                if let Some(civ_raw) = r.players[slot as usize].civ_raw {
                                    r.players[slot as usize].imperialtime = Some(
                                        r.duration
                                            + match civ_raw {
                                                8 => 190000 / 1.10 as u32,
                                                _ => 190000,
                                            },
                                    )
                                } else {
                                    r.players[slot as usize].imperialtime = Some(r.duration + 190000)
                                }
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
                        const MOVE_CMD_SIZE: usize = 19;
                        if r.debug.earlymovecount < EARLYMOVE_THRESHOLD && b.remain() >= MOVE_CMD_SIZE {
                            r.debug.earlymovecmd.push(b.current()[..MOVE_CMD_SIZE].try_into()?);
                            r.debug.earlymovetime.push(r.duration);
                            r.debug.earlymovecount += 1;
                        }
                    }
                    COMMAND_SAVE => {
                        // Handle save command
                    }
                    COMMAND_CHAPTER => {
                        // Handle chapter command
                    }
                    _ => {
                        // Handle unknown command
                    }
                }

                b.seek(nextpos);
            }
            OP_SYNC => {
                let time_delta = val!(b.get_i32());
                if time_delta < 0 || time_delta > 1000 {
                    #[cfg(debug_assertions)]
                    bail!("Unusual time delta: {} @bodypos: {}", time_delta, b.tell() - 4);
                    #[allow(unreachable_code)]
                    continue;
                }
                r.duration += time_delta as u32;
                let sync_data = val!(b.get_i32());
                b.mov(if sync_data != 0x03 { 28 } else { 0 });
                b.mov(12);
            }
            OP_VIEWLOCK => {
                b.mov(12);
            }
            OP_CHAT => {
                let command = val!(b.get_i32());
                if command == 500 {
                    if r.ver == Some(Version::AoK) || r.ver == Some(Version::AoKTrial) {
                        b.mov(32);
                    } else {
                        b.mov(20);
                    }
                    continue;
                }
                debug_assert_eq!(command, -1);
                let msg = b.extract_str_l32();
                if let Some(message) = msg.as_ref() {
                    if message.len() >= 7
                        && message.starts_with(b"@#")
                        && message.ends_with(b"--")
                        && message[3] == b'-'
                        && message[4] == b'-'
                        || message.len() == 0
                    {
                        continue;
                    }

                    r.chat.push(Chat { time: Some(r.duration), player: None, content_raw: msg, content: None });
                }
            }
            _ => {
                #[cfg(debug_assertions)]
                if r.ver == Some(Version::AoK)
                    || r.ver == Some(Version::AoKTrial)
                    || r.ver == Some(Version::AoC)
                    || r.ver == Some(Version::AoCTrial)
                    || r.ver == Some(Version::AoC10a)
                    || r.ver == Some(Version::AoC10c)
                {
                    bail!("Unknown Operation: {} @ {}", op_type, b.tell() - 4);
                }
            }
        }
    }

    Ok(())
}
