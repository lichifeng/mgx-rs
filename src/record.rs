use serde::Serialize;
use std::fmt::Debug;

/// Get value from `Option<T>` if it's Some and bail if it's None
#[doc(hidden)]
#[macro_export]
macro_rules! val {
    ($x:expr) => {
        match $x {
            Some(x) => x,
            None => bail!("{:?} is None", stringify!($x)),
        }
    };
}

/// Store information of this game extracted from the recorded game. Most fields will be `None` if not present in the recorded game or exception occurs during parsing
#[derive(Debug, Serialize)]
pub struct Record {
    pub filename: String,
    pub filesize: usize,
    pub lastmod: u64,
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
    pub totalplayers: Option<u8>,
    pub mapsize_raw: Option<u32>,
    pub mapsize: Option<String>,
    pub revealmap_raw: Option<i32>,
    pub revealmap: Option<String>,
    pub mapx: Option<i32>,
    pub mapy: Option<i32>,
    pub nofog: Option<bool>,
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
    pub players: [Player; 9],
    /// Debug data used by the parser. Strip this out in output json.
    #[serde(skip)]
    pub debug: DebugInfo,
}

/// Information of a player
#[derive(Debug, Serialize)]
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
    pub modversion: Option<f32>,
    pub winner: Option<bool>,
}

impl Player {
    pub fn new(slot: usize) -> Self {
        Player {
            slot,
            index: None,
            playertype: None,
            name_raw: None,
            name: None,
            teamid: None,
            ismainop: None,
            initx: None,
            inity: None,
            civ_raw: None,
            civ: None,
            colorid: None,
            disconnected: None,
            resigned: None,
            feudaltime: None,
            castletime: None,
            imperialtime: None,
            initage_raw: None,
            initage: None,
            initfood: None,
            initwood: None,
            initstone: None,
            initgold: None,
            initpop: None,
            initcivilian: None,
            initmilitary: None,
            modversion: None,
            winner: None,
        }
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
#[derive(Debug)]
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
    pub fn new(filename: String, filesize: usize, lastmod: u64) -> Self {
        Record {
            filename,
            filesize,
            lastmod,
            guid: None,
            verlog: None,
            ver: None,
            verraw: None,
            versave: None,
            versave2: None,
            verscenario: None,
            include_ai: None,
            speed_raw: None,
            speed: None,
            recorder: None,
            totalplayers: None,
            mapsize_raw: None,
            mapsize: None,
            revealmap_raw: None,
            revealmap: None,
            mapx: None,
            mapy: None,
            nofog: None,
            instantbuild: None,
            enablecheats: None,
            restoretime: None,
            ismultiplayer: None,
            isconquest: None,
            relics2win: None,
            explored2win: None,
            anyorall: None, // TODO: what's this? name it to `all2win`?
            victorytype_raw: None,
            victorytype: None,
            score2win: None,
            time2win_raw: None,
            time2win: None,
            scenariofilename_raw: None,
            scenariofilename: None,
            instructions_raw: None,
            instructions: None,
            duration: 0,
            chat: Vec::new(),
            mapid: None,
            mapname: None,
            difficulty_raw: None,
            difficulty: None,
            lockteams: None,
            poplimit: None,
            gametype_raw: None,
            gametype: None,
            lockdiplomacy: None,
            haswinner: false,
            matchup: None,
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
                currentpos_header: 0,
                currentpos_body: 0,
                aipos: 0,
                initpos: 0,
                triggerpos: 0,
                triggersign: 1.6, // Other values in higher versions
                settingspos: 0,
                disabledtechspos: 0,
                victorypos: 0,
                scenariopos: 0,
                mappos: None,
                playerinitpos_by_idx: [None, None, None, None, None, None, None, None, None],
                earlymovecount: 0,
                earlymovecmd: Vec::new(),
                earlymovetime: Vec::new(),
            },
        }
    }
}
