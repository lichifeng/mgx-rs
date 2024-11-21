use std::fmt::Debug;

#[derive(Debug)]
pub struct Record {
    pub filename: Option<String>,
    pub filesize: Option<usize>,
    pub lastmod: Option<u64>,
    pub vercode: Option<String>,
    pub verstr: Option<String>,
    pub versave: Option<f32>,
    pub versave2: Option<u32>,
    pub include_ai: Option<bool>,
    pub speed: Option<u32>,
    pub recorder: Option<u16>,
    pub totalplayers: Option<u8>,
    pub mapsize: Option<i32>,
    pub revealmap: Option<i32>,
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
    pub victorymode: Option<i32>,
    pub score2win: Option<i32>,
    pub time2win: Option<i32>,
    pub scenariofilename: Option<Vec<u8>>,
    pub instructions: Option<Vec<u8>>,
    pub duration: u32,
    pub chat: Vec<Chat>,
    pub mapid: Option<i32>,
    pub difficultyid: Option<i32>,
    pub lockteams: Option<bool>,
    pub poplimit: Option<i32>,
    pub gametype: Option<u8>,
    pub lockdiplomacy: Option<bool>,
    pub players: [Player; 9],
    pub debug: DebugInfo,
}

#[derive(Debug)]
pub struct Player {
    pub slot: usize,
    pub index: Option<i32>,
    pub playertype: Option<i32>,
    pub name: Option<Vec<u8>>,
    pub teamid: Option<u8>,
    pub ismainop: Option<bool>,
    pub initx: Option<f32>,
    pub inity: Option<f32>,
    pub civid: Option<u8>,
    pub colorid: Option<u8>,
}

impl Player {
    pub fn new(slot: usize) -> Self {
        Player {
            slot,
            index: None,
            playertype: None,
            name: None,
            teamid: None,
            ismainop: None,
            initx: None,
            inity: None,
            civid: None,
            colorid: None,
        }
    }

    pub fn isvalid(&self) -> bool {
        self.playertype.is_some_and(|x| x >= 2 && x <= 5)
    }
}

#[derive(Debug)]
pub struct Chat {
    pub time: Option<u32>,
    pub player: Option<u8>,
    pub message: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct DebugInfo {
    pub currentpos_header: usize,
    pub currentpos_body: usize,
    pub headerstart: u32,
    pub headerend: u32,
    pub nextpos: u32,
    pub headerlen: usize,
    pub aipos: usize,
    pub mappos: usize,
    pub initpos: usize,
    pub triggerpos: usize,
    pub triggersign: f64,
    pub settingspos: usize,
    pub disabledtechspos: usize,
    pub victorypos: usize,
    pub scenariopos: usize,
    pub lobbypos: usize,
    pub playerinitpos_by_idx: [Option<usize>; 9],
    pub earlymovecount: usize,
}

impl Record {
    pub fn new() -> Self {
        Record {
            filename: None,
            filesize: None,
            lastmod: None,
            vercode: None,
            verstr: None,
            versave: None,
            versave2: None,
            include_ai: None,
            speed: None,
            recorder: None,
            totalplayers: None,
            mapsize: None,
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
            victorymode: None,
            score2win: None,
            time2win: None,
            scenariofilename: None,
            instructions: None,
            duration: 0,
            chat: Vec::new(),
            mapid: None,
            difficultyid: None,
            lockteams: None,
            poplimit: None,
            gametype: None,
            lockdiplomacy: None,
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
                headerstart: 0,
                headerend: 0,
                nextpos: 0,
                headerlen: 0,
                aipos: 0,
                mappos: 0,
                initpos: 0,
                triggerpos: 0,
                triggersign: 1.6, // Other values in higher versions
                settingspos: 0,
                disabledtechspos: 0,
                victorypos: 0,
                scenariopos: 0,
                lobbypos: 0,
                playerinitpos_by_idx: [None, None, None, None, None, None, None, None, None],
                earlymovecount: 0,
            },
        }
    }
}
