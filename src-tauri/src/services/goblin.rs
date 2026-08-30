use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::services::garmin::GarminBiometrics;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoblinProfile {
    pub id: i64,
    pub name: String,
    pub level: i32,
    pub xp: i32,
    pub xp_to_next_level: i32,
    pub gold: i32,
    pub evolution_rank: String,
    pub mood: String,
    pub energy: i32,
    pub hunger: i32,
    pub streak_days: i32,
    pub last_active: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoblinQuest {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub quest_category: String,
    pub target_metric: String,
    pub target_value: f64,
    pub current_value: f64,
    pub reward_xp: i32,
    pub reward_gold: i32,
    pub is_completed: bool,
    pub is_claimed: bool,
    pub expires_at: Option<String>,
}

pub fn get_goblin_profile(db: Arc<Mutex<Connection>>) -> Result<GoblinProfile, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, level, xp, xp_to_next_level, gold, evolution_rank,
                    mood, energy, hunger, streak_days, last_active
             FROM goblin_profile WHERE id = 1",
        )
        .map_err(|e| e.to_string())?;

    let profile = stmt
        .query_row([], |row| {
            Ok(GoblinProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                level: row.get(2)?,
                xp: row.get(3)?,
                xp_to_next_level: row.get(4)?,
                gold: row.get(5)?,
                evolution_rank: row.get(6)?,
                mood: row.get(7)?,
                energy: row.get(8)?,
                hunger: row.get(9)?,
                streak_days: row.get(10)?,
                last_active: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(profile)
}

pub fn award_xp_and_gold(
    conn: &Connection,
    added_xp: i32,
    added_gold: i32,
) -> Result<GoblinProfile, String> {
    let mut current = get_goblin_profile_internal(conn)?;
    current.xp += added_xp;
    current.gold += added_gold;

    // Check level up logic
    while current.xp >= current.xp_to_next_level {
        current.xp -= current.xp_to_next_level;
        current.level += 1;
        current.xp_to_next_level = ((current.level as f64).powf(1.4) * 400.0) as i32 + 100;
        current.evolution_rank = determine_evolution_rank(current.level);
        current.mood = "Proud".to_string();
    }

    conn.execute(
        "UPDATE goblin_profile SET 
            level = ?1, xp = ?2, xp_to_next_level = ?3, gold = ?4, 
            evolution_rank = ?5, mood = ?6, last_active = datetime('now')
         WHERE id = 1",
        (
            current.level,
            current.xp,
            current.xp_to_next_level,
            current.gold,
            &current.evolution_rank,
            &current.mood,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(current)
}

pub fn add_xp_and_gold(
    db: Arc<Mutex<Connection>>,
    added_xp: i32,
    added_gold: i32,
) -> Result<GoblinProfile, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;
    award_xp_and_gold(&conn, added_xp, added_gold)
}

fn determine_evolution_rank(level: i32) -> String {
    match level {
        1..=3 => "Cave Scamp".to_string(),
        4..=7 => "Cavern Scout".to_string(),
        8..=12 => "Dungeon Raider".to_string(),
        13..=18 => "Goblin Shaman".to_string(),
        19..=25 => "Mountain Behemoth".to_string(),
        _ => "Goblin Warlord".to_string(),
    }
}

pub fn update_goblin_mood_from_biometrics(
    db: Arc<Mutex<Connection>>,
    bio: &GarminBiometrics,
) -> Result<String, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let calculated_mood = if bio.body_battery < 25 {
        "Exhausted".to_string()
    } else if bio.stress_level > 65 {
        "Feral".to_string()
    } else if bio.sleep_score >= 85 && bio.hrv_status == "Balanced" {
        "Zen".to_string()
    } else if bio.steps >= bio.step_goal {
        "Proud".to_string()
    } else if bio.body_battery > 65 {
        "Energetic".to_string()
    } else {
        "Neutral".to_string()
    };

    let energy = bio.body_battery;
    let hunger = (bio.active_calories / 15).min(100);

    conn.execute(
        "UPDATE goblin_profile SET mood = ?1, energy = ?2, hunger = ?3 WHERE id = 1",
        (&calculated_mood, energy, hunger),
    )
    .map_err(|e| e.to_string())?;

    Ok(calculated_mood)
}

pub fn get_quests(db: Arc<Mutex<Connection>>) -> Result<Vec<GoblinQuest>, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, quest_category, target_metric, target_value,
                    current_value, reward_xp, reward_gold, is_completed, is_claimed, expires_at
             FROM goblin_quests ORDER BY is_claimed ASC, is_completed DESC, id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(GoblinQuest {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                quest_category: row.get(3)?,
                target_metric: row.get(4)?,
                target_value: row.get(5)?,
                current_value: row.get(6)?,
                reward_xp: row.get(7)?,
                reward_gold: row.get(8)?,
                is_completed: row.get(9)?,
                is_claimed: row.get(10)?,
                expires_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut quests = Vec::new();
    for r in rows {
        quests.push(r.map_err(|e| e.to_string())?);
    }
    Ok(quests)
}

pub fn update_quest_progress(
    db: Arc<Mutex<Connection>>,
    bio: &GarminBiometrics,
) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, target_metric, target_value, current_value, is_completed, is_claimed
             FROM goblin_quests WHERE is_claimed = 0",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, bool>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut updates = Vec::new();
    for (id, metric, target, mut current, mut completed, _) in rows.flatten() {
        match metric.as_str() {
            "steps" => current = bio.steps as f64,
            "sleep_score" => current = bio.sleep_score as f64,
            "intensity_minutes" => current = bio.intensity_minutes as f64,
            "active_calories" => current = bio.active_calories as f64,
            "stress_level" => {
                // For stress, lower than target is good
                if (bio.stress_level as f64) <= target {
                    completed = true;
                }
                current = bio.stress_level as f64;
            }
            _ => {}
        }

        if metric != "stress_level" && current >= target {
            completed = true;
        }

        updates.push((id, current, completed));
    }

    for (id, curr, comp) in updates {
        conn.execute(
            "UPDATE goblin_quests SET current_value = ?1, is_completed = ?2 WHERE id = ?3",
            rusqlite::params![curr, comp, id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn claim_quest_reward(
    db: Arc<Mutex<Connection>>,
    quest_id: i64,
) -> Result<GoblinProfile, String> {
    let (reward_xp, reward_gold) = {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        let (is_completed, is_claimed, xp, gold): (bool, bool, i32, i32) = conn
            .query_row(
                "SELECT is_completed, is_claimed, reward_xp, reward_gold FROM goblin_quests WHERE id = ?1",
                [quest_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| e.to_string())?;

        if !is_completed {
            return Err("Quest is not completed yet!".to_string());
        }
        if is_claimed {
            return Err("Quest reward has already been claimed!".to_string());
        }

        conn.execute(
            "UPDATE goblin_quests SET is_claimed = 1 WHERE id = ?1",
            [quest_id],
        )
        .map_err(|e| e.to_string())?;

        (xp, gold)
    };

    add_xp_and_gold(db, reward_xp, reward_gold)
}

fn get_goblin_profile_internal(conn: &Connection) -> Result<GoblinProfile, String> {
    conn.query_row(
        "SELECT id, name, level, xp, xp_to_next_level, gold, evolution_rank,
                mood, energy, hunger, streak_days, last_active
         FROM goblin_profile WHERE id = 1",
        [],
        |row| {
            Ok(GoblinProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                level: row.get(2)?,
                xp: row.get(3)?,
                xp_to_next_level: row.get(4)?,
                gold: row.get(5)?,
                evolution_rank: row.get(6)?,
                mood: row.get(7)?,
                energy: row.get(8)?,
                hunger: row.get(9)?,
                streak_days: row.get(10)?,
                last_active: row.get(11)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}
