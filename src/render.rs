use crate::trans;
use crate::translations::{en, zh};
use crate::Record;
use anyhow::Result;
use encoding_rs::Encoding;
use phf::phf_map;

static ENCODING_MAP: phf::Map<&'static [u8], &'static str> = phf_map! {
    b"\xb5\xd8\xcd\xbc\xc0\xe0\xb1\xf0" => "GBK",           // kZh
    b"\xb5\xd8\xcd\xbc\xc0\xe0\xd0\xcd" => "GBK",
    b"\xa6\x61\xb9\xcf\xc3\xfe\xa7\x4f" => "Big5",          // kZhTw
    b"\xa6\x61\xb9\xcf\xc3\xfe\xab\xac" => "Big5",
    b"\x83\x7d\x83\x62\x83\x76\x82\xcc\x8e\xed\x97\xde" => "Shift_JIS",
    b"\xc1\xf6\xb5\xb5\x20\xc1\xbe\xb7\xf9" => "EUC-KR",
    b"Tipo de Mapa" => "windows-1252",    // kBr
    b"Kartentyp" => "windows-1252",       // kDe
    b"Map Type" => "windows-1252",        // kEn
    b"Tipo de mapa" => "windows-1252",    // kEs
    b"Type de carte" => "windows-1252",   // kFr
    b"Tipo di mappa" => "windows-1252",   // kIt
    b"Kaarttype" => "windows-1252",       // kNl
    b"\xd2\xe8\xef\x20\xea\xe0\xf0\xf2\xfb" => "windows-1251",
    b"\xd0\xa2\xd0\xb8\xd0\xbf\x20\xd0\xba\xd0\xb0\xd1\x80\xd1\x82\xd1\x8b" => "windows-1251",
    b"\xe5\x9c\xb0\xe5\x9b\xbe\xe7\xb1\xbb\xe5\x9e\x8b" => "UTF-8",  // kZhUtf8
    b"\xe3\x83\x9e\xe3\x83\x83\xe3\x83\x97\xe3\x83\x81\xe7\xa8\xae\xe9\xa1\x9e" => "UTF-8",
    b"\xec\xa7\x80\xeb\x8f\x84\x20\xec\xa2\x85\xeb\xa5\x98" => "UTF-8"
};

impl Record {
    pub fn detect_encoding(&self) -> Option<String> {
        let instruction = self.instructions_raw.as_ref()?;
        ENCODING_MAP.entries().find_map(|(pattern, encoding)| {
            instruction.windows(pattern.len()).any(|window| window == *pattern).then(|| encoding.to_string())
        })
    }

    pub fn translate(&mut self, lang: &str) {
        self.gametype = trans!(self.gametype_raw, lang, GAME_TYPES_TRANS);
        self.difficulty = trans!(self.difficulty_raw, lang, DIFFICULTIES_TRANS);
        self.revealmap = trans!(self.revealmap_raw, lang, REVEAL_MAP_TRANS);
        self.mapsize = trans!(self.mapsize_raw, lang, MAP_SIZES_TRANS);
        self.speed = trans!(self.speed_raw, lang, GAME_SPEEDS_TRANS);
        self.victorytype = trans!(self.victorytype_raw, lang, VICTORY_TYPE_TRANS);
        self.time2win = trans!(self.time2win_raw, lang, VICTORY_TIME_TRANS);
        self.mapname = trans!(self.mapid, lang, MAP_NAMES_TRANS);
        for p in self.players.iter_mut() {
            p.civ = trans!(p.civ_raw, lang, CIVILIZATIONS_TRANS);
            p.initage = trans!(p.initage_raw, lang, AGES_TRANS);
        }
    }

    pub fn convert_encoding(&mut self) {
        let encoding_name = self.detect_encoding().unwrap_or_else(|| "GBK".to_string());
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(encoding_rs::GBK);

        match self.instructions_raw.as_ref() {
            Some(x) => {
                let (decoded, _, _) = encoding.decode(x);
                self.instructions = Some(decoded.into_owned());
            }
            None => (),
        }

        for p in &mut self.players {
            match p.name_raw.as_ref() {
                Some(x) => {
                    let (decoded, _, _) = encoding.decode(x);
                    p.name = Some(decoded.into_owned());
                }
                None => (),
            }
        }

        for c in &mut self.chat {
            match c.content_raw.as_ref() {
                Some(x) => {
                    let (decoded, _, _) = encoding.decode(x);
                    c.content = Some(decoded.into_owned());
                }
                None => (),
            }
        }
    }

    pub fn dump_json(&mut self) -> Result<String> {
        self.convert_encoding();
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Translates a raw value to a human-readable string
#[doc(hidden)]
#[macro_export]
macro_rules! trans {
    ($raw:expr, $lang:expr, $string:ident) => {
        if let Some(x) = $raw.as_ref() {
            let translated = match $lang {
                "en" => en::$string.get(&(*x as i32)),
                _ => zh::$string.get(&(*x as i32)),
            };
            if let Some(y) = translated {
                Some(y.to_string())
            } else {
                None
            }
        } else {
            None
        }
    };
}
