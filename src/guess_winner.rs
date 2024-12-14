use crate::{Record, Player};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Only games with 2 sides are evaluated.
/// A team is considered to have won if players of the other side all have resigned.
/// If all players of PoV's team have resigned and the other side has survivors, the other side is considered to have won.
/// Matchup is generated here, too.
pub fn guess(rec: &mut Record) -> Result<()> {
    let mut teams: HashMap<u8, Vec<&mut Player>> = HashMap::new();

    // teamid == 1 means no team
    for p in rec.players.iter_mut() {
        if !p.isvalid() {
            continue;
        }

        let teamid = match p.teamid {
            Some(id) => {
                if id > 1 {
                    id
                } else {
                    100 + p.slot as u8
                }
            }
            _ => continue,
        };

        if teams.contains_key(&teamid) {
            if let Some(members) = teams.get_mut(&teamid) {
                members.push(p);
            }
        } else {
            let members = vec![p];
            teams.insert(teamid, members);
        }
    }

    let mut team_sizes: Vec<usize> = teams.values().map(|v| v.len()).collect();
    team_sizes.sort();
    rec.matchup = Some(team_sizes);

    if teams.len() != 2 {
        return Ok(()); // not a 2-sided game
    }

    let mut iter = teams.iter_mut();
    let (team1_id, team1) = iter.next().ok_or(anyhow!("No team 1"))?;
    let (team2_id, team2) = iter.next().ok_or(anyhow!("No team 2"))?;

    if team1.len() != team2.len() {
        return Ok(()); // not a fair game
    }

    let mut pov_team: Option<&u8> = Option::None;
    let mut pov_resigned = false;
    let mut team1_resigned: usize = 0;
    let mut team2_resigned: usize = 0;

    team1.iter().for_each(|p| {
        if p.resigned.is_some() {
            team1_resigned += 1;
        }
        if let Some(recorder_slot) = rec.recorder {
            if p.slot == recorder_slot as usize {
                pov_team = Some(team1_id);
                pov_resigned = p.resigned.is_some();
            }
        }
    });

    team2.iter().for_each(|p| {
        if p.resigned.is_some() {
            team2_resigned += 1;
        }
        if let Some(recorder_slot) = rec.recorder {
            if p.slot == recorder_slot as usize {
                pov_team = Some(team2_id);
                pov_resigned = p.resigned.is_some();
            }
        }
    });

    if team1_resigned == team1.len() && !(team2_resigned == team2.len()) {
        for p in team2.iter_mut() {
            p.winner = Some(true);
            rec.haswinner = true;
        }
    } else if team2_resigned == team2.len() && !(team1_resigned == team1.len()) {
        for p in team1.iter_mut() {
            p.winner = Some(true);
            rec.haswinner = true;
        }
    } else if Some(team1_id) == pov_team && team1_resigned == team1.len() - 1 {
        for p in team2.iter_mut() {
            p.winner = Some(true);
            rec.haswinner = true;
        }
    } else if Some(team2_id) == pov_team && team2_resigned == team2.len() - 1 {
        for p in team1.iter_mut() {
            p.winner = Some(true);
            rec.haswinner = true;
        }
    }

    Ok(())
}
