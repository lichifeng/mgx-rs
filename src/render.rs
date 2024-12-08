use crate::record::Record;
use encoding_rs::Encoding;
use phf::phf_map;
use anyhow::Result;

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
    b"Type de carte" => "windows-1252",  // kFr
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

    pub fn dump_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}
