use crate::Record;
use anyhow::Result;

/// Only games with 2 sides are evaluated.
/// A team is considered to have won if players of the other side all have resigned.
/// If all players of PoV's team have resigned and the other side has survivors, the other side is considered to have won.
/// Matchup is generated here, too.
pub fn guess(rec: &mut Record) -> Result<()> {
    if !rec.matchup.is_some() {
        return Ok(());
    }

    // if not 2-sided game or not a fair game, ignore
    if rec.teams.len() != 2 || rec.teams[0].len() != rec.teams[1].len() {
        return Ok(());
    }

    let mut all_resigned_0 = true;
    let mut all_resigned_1 = true;
    // test resigned info in the first team
    for idx in &rec.teams[0] {
        if !all_resigned_0 {
            break;
        }
        for p in &rec.players {
            if p.index == Some(*idx) && p.resigned.is_none() {
                all_resigned_0 = false;
                break;
            }
        }
    }
    // test resigned info in the second team
    for idx in &rec.teams[1] {
        if !all_resigned_1 {
            break;
        }
        for p in &rec.players {
            if p.index == Some(*idx) && p.resigned.is_none() {
                all_resigned_1 = false;
                break;
            }
        }
    }

    if all_resigned_0 == all_resigned_1 {
        rec.haswinner = false;
        return Ok(());
    } else {
        rec.haswinner = true;
    }

    let winner_team = if all_resigned_0 {
        &rec.teams[1]
    } else {
        &rec.teams[0]
    };

    rec.players.iter_mut().for_each(|p| {
        if p.index.as_ref().map_or(false, |idx| winner_team.contains(idx)) {
            p.winner = Some(true);
        } else {
            p.winner = Some(false);
        }
    });

    Ok(())
}
