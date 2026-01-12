use crate::Record;
use anyhow::Result;

/// Only games with 2 sides are evaluated.
/// A team is considered to have won if players of the other side all have resigned.
/// If all players of PoV's team have resigned and the other side has survivors, the other side is considered to have won.
/// Matchup is generated here, too.
pub fn guess(rec: &mut Record) -> Result<()> {
    if rec.matchup.is_none() {
        return Ok(());
    }

    if rec.instantbuild == Some(true) || rec.enablecheats == Some(true) {
        return Ok(());
    }

    // if not 2-sided game or not a fair game, ignore
    if rec.teams.len() != 2 || rec.teams[0].len() != rec.teams[1].len() {
        return Ok(());
    }

    let mut resigned: Vec<i32> = vec![];
    rec.players.iter().for_each(|p| {
        if !p.isvalid() {
            return;
        }

        if let Some(idx) = p.index {
            if let Some(resign_time) = p.resigned {
                if resign_time > 0 {
                    resigned.push(idx);
                }
            }
        }
    });

    // If no one has resigned data, usually means the recorder is the only one who resigned
    if resigned.is_empty() {
        if let Some(pov) = rec.recorder {
            resigned.push(pov as i32);
        } else {
            return Ok(());
        }
    }

    let winner_team: &Vec<i32>;
    if is_subset(&resigned, &rec.teams[0]) {
        winner_team = &rec.teams[1];
    } else if is_subset(&resigned, &rec.teams[1]) {
        winner_team = &rec.teams[0];
    } else {
        // Cannot determine winner
        return Ok(());
    }

    rec.players.iter_mut().for_each(|p| {
        if p.index.as_ref().is_some_and(|idx| winner_team.contains(idx)) {
            p.winner = Some(true);
        } else {
            p.winner = Some(false);
        }
    });

    rec.haswinner = true;

    Ok(())
}

// Test if a Vec<i32> is a subset of another Vec<i32>
fn is_subset(sub: &[i32], sup: &[i32]) -> bool {
    sub.iter().all(|item| sup.contains(item))
}
