use serde::Serialize;
use std::fmt::Debug;

/// Get value from `Option<T>` if it's Some and `anyhow::bail!` if `None`
#[doc(hidden)]
#[macro_export]
macro_rules! val {
    ($x:expr) => {
        match $x {
            Some(x) => x,
            None => bail!("{:?} is None @ {}, line: {}", stringify!($x), file!(), line!()),
        }
    };
}

/// Store information of this game extracted from the recorded game. Most fields will be `None` if not present in the recorded game or exception occurs during parsing
#[derive(Debug, Serialize, Default)]
pub struct Record {
    pub parser: String,
    pub md5: Option<String>,
    pub filename: String,
    pub filesize: usize,
    /// In milliseconds since epoch
    pub lastmod: u128,
    pub guid: Option<String>,
    pub verlog: Option<u32>,
    pub ver: Option<Version>,
    pub verraw: Option<String>,
    /// `11.76` for aoc10a/c
    pub versave: Option<f32>,
    pub versave2: Option<u32>,
    pub verscenario: Option<f32>,
    pub include_ai: Option<bool>,
    pub speed_raw: Option<u32>,
    pub speed: Option<String>,
    pub recorder: Option<u16>,
    /// **GAIA** is counted
    pub totalplayers: Option<u8>,
    pub mapsize_raw: Option<u32>,
    pub mapsize: Option<String>,
    pub revealmap_raw: Option<i32>,
    pub revealmap: Option<String>,
    pub mapx: Option<i32>,
    pub mapy: Option<i32>,
    pub fogofwar: Option<bool>,
    pub instantbuild: Option<bool>,
    pub enablecheats: Option<bool>,
    pub restoretime: Option<u32>,
    pub ismultiplayer: Option<bool>,
    pub isconquest: Option<bool>,
    pub relics2win: Option<i32>,
    pub explored2win: Option<i32>,
    pub anyorall: Option<bool>,
    pub victorytype_raw: Option<i32>,
    pub victorytype: Option<String>,
    pub score2win: Option<i32>,
    pub time2win_raw: Option<i32>,
    pub time2win: Option<String>,
    #[serde(skip)]
    pub scenariofilename_raw: Option<Vec<u8>>,
    pub scenariofilename: Option<String>,
    #[serde(skip)]
    pub instructions_raw: Option<Vec<u8>>,
    pub instructions: Option<String>,
    pub duration: u32,
    pub chat: Vec<Chat>,
    pub mapid: Option<u32>,
    pub mapname: Option<String>,
    pub difficulty_raw: Option<i32>,
    pub difficulty: Option<String>,
    pub lockteams: Option<bool>,
    pub poplimit: Option<u32>,
    pub gametype_raw: Option<u8>,
    pub gametype: Option<String>,
    pub lockdiplomacy: Option<bool>,
    pub haswinner: bool,
    pub matchup: Option<Vec<usize>>,
    pub teams: Vec<Vec<i32>>,
    pub players: [Player; 9],
    /// Debug data used by the parser. Strip this out in output json.
    #[serde(skip)]
    pub debug: DebugInfo,
}

/// Information of a player
#[derive(Debug, Serialize, Default)]
pub struct Player {
    pub slot: usize,
    pub index: Option<i32>,
    pub playertype: Option<i32>,
    #[serde(skip)]
    pub name_raw: Option<Vec<u8>>,
    pub name: Option<String>,
    pub teamid: Option<u8>,
    pub ismainop: Option<bool>,
    pub initx: Option<f32>,
    pub inity: Option<f32>,
    pub civ_raw: Option<u8>,
    pub civ: Option<String>,
    pub colorid: Option<u8>,
    pub disconnected: Option<bool>,
    pub resigned: Option<u32>,
    pub feudaltime: Option<u32>,
    pub castletime: Option<u32>,
    pub imperialtime: Option<u32>,
    pub initage_raw: Option<f32>,
    pub initage: Option<String>,
    pub initfood: Option<f32>,
    pub initwood: Option<f32>,
    pub initstone: Option<f32>,
    pub initgold: Option<f32>,
    pub initpop: Option<f32>,
    pub initcivilian: Option<f32>,
    pub initmilitary: Option<f32>,
    /// Only presents in UP1.5
    pub modversion: Option<f32>,
    /// Default is `false`. Only for fair 2-sided games
    pub winner: Option<bool>,
}

impl Player {
    pub fn new(slot: usize) -> Self {
        Player { slot, ..Default::default() }
    }

    pub fn isvalid(&self) -> bool {
        self.playertype.is_some_and(|x| x >= 2 && x <= 5)
    }
}

/// Information of a chat message. Lobby chats don't have time. Field `player` is not implemented yet
#[derive(Debug, Serialize)]
pub struct Chat {
    pub time: Option<u32>,
    /// Not implemented yet
    pub player: Option<u8>,
    #[serde(skip)]
    pub content_raw: Option<Vec<u8>>,
    pub content: Option<String>,
}

/// Debug information used by the parser
#[derive(Debug, Default)]
pub struct DebugInfo {
    pub currentpos_header: usize,
    pub currentpos_body: usize,
    pub aipos: usize,
    pub initpos: usize,
    pub triggerpos: usize,
    pub triggersign: f64,
    pub settingspos: usize,
    pub disabledtechspos: usize,
    pub victorypos: usize,
    pub scenariopos: usize,
    pub mappos: Option<usize>,
    pub playerinitpos_by_idx: [Option<usize>; 9],
    pub earlymovecount: usize,
    pub earlymovecmd: Vec<[u8; 19]>,
    pub earlymovetime: Vec<u32>,
}

/// Version of the recorded game
#[derive(Debug, PartialEq, Serialize)]
pub enum Version {
    AoKTrial,
    AoK,
    AoCTrial,
    AoC,
    AoC10a,
    AoC10c,
    UP12,
    UP13,
    UP14,
    UP14RC1,
    UP14RC2,
    UP15,
    AoFE21,
    HD,
    DE,
    MCP,
    Unknown,
}

impl Record {
    pub fn new(filename: String, filesize: usize, lastmod: u128) -> Self {
        Record {
            parser: format!(
                "mgx-rs {}-{}",
                env!("CARGO_PKG_VERSION"),
                if cfg!(debug_assertions) { "debug" } else { "release" }
            ),
            filename,
            filesize,
            lastmod,
            players: [
                Player::new(0),
                Player::new(1),
                Player::new(2),
                Player::new(3),
                Player::new(4),
                Player::new(5),
                Player::new(6),
                Player::new(7),
                Player::new(8),
            ],
            debug: DebugInfo {
                triggersign: 1.6, // Other values in higher versions
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
