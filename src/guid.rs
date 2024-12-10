use crate::Record;
use anyhow::Result;
use chksum_hash_md5 as md5;

pub fn calc_guid(rec: &Record) -> Result<String> {
    let mut hasher = md5::default();

    if let Some(verraw) = rec.verraw.as_ref() {
        hasher.update(verraw.as_bytes());
    }
    if let Some(versave) = rec.versave {
        hasher.update(versave.to_le_bytes());
    }
    if let Some(verlog) = rec.verlog {
        hasher.update(verlog.to_le_bytes());
    }
    if let Some(verscenario) = rec.verscenario {
        hasher.update(verscenario.to_le_bytes());
    }
    if let Some(mapsize_raw) = rec.mapsize_raw {
        hasher.update(mapsize_raw.to_le_bytes());
    }
    if let Some(poplimit) = rec.poplimit {
        hasher.update(poplimit.to_le_bytes());
    }
    if let Some(speed_raw) = rec.speed_raw {
        hasher.update(speed_raw.to_le_bytes());
    }
    if let Some(mapid) = rec.mapid {
        hasher.update(mapid.to_le_bytes());
    }

    for cmd in rec.debug.earlymovecmd.iter() {
        hasher.update(cmd);
    }

    for t in rec.debug.earlymovetime.iter() {
        hasher.update(t.to_le_bytes());
    }

    for p in rec.players.iter() {
        if let Some(name_raw) = p.name_raw.as_ref() {
            hasher.update(name_raw.as_slice());
        }
        if let Some(civ_raw) = p.civ_raw {
            hasher.update(civ_raw.to_le_bytes());
        }
        if let Some(index) = p.index {
            hasher.update(index.to_le_bytes());
        }
        hasher.update(p.slot.to_le_bytes());
        if let Some(colorid) = p.colorid {
            hasher.update(colorid.to_le_bytes());
        }
        if let Some(teamid) = p.teamid {
            hasher.update(teamid.to_le_bytes());
        }
    }

    Ok(hasher.digest().to_hex_lowercase())
}
