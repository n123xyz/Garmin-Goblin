use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::services::garmin::GarminBiometrics;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthInsight {
    pub id: i64,
    pub created_at: String,
    pub category: String,
    pub readiness_score: i32,
    pub readiness_state: String,
    pub clinical_summary: String,
    pub goblin_reaction: String,
    pub actionable_tips: String,
}

pub fn calculate_readiness(bio: &GarminBiometrics) -> (i32, String) {
    let mut score: f64 = 0.0;

    // Body Battery contribution (35%)
    score += (bio.body_battery as f64).clamp(0.0, 100.0) * 0.35;

    // Sleep Score contribution (30%)
    score += (bio.sleep_score as f64).clamp(0.0, 100.0) * 0.30;

    // HRV status contribution (20%)
    let hrv_score = match bio.hrv_status.as_str() {
        "Balanced" => 90.0,
        "Unbalanced" => 55.0,
        "Low" => 30.0,
        _ => 75.0,
    };
    score += hrv_score * 0.20;

    // Stress inverse contribution (15%)
    let stress_score = (100.0 - bio.stress_level as f64).clamp(0.0, 100.0);
    score += stress_score * 0.15;

    let final_score = score.round() as i32;
    let state = match final_score {
        85..=100 => "Peak Readiness".to_string(),
        70..=84 => "Optimal Readiness".to_string(),
        50..=69 => "Moderate / Maintenance".to_string(),
        30..=49 => "Fatigued / Active Recovery".to_string(),
        _ => "High Strain / Rest Demanded".to_string(),
    };

    (final_score, state)
}

pub fn get_latest_health_insights(
    db: Arc<Mutex<Connection>>,
    limit: usize,
) -> Result<Vec<HealthInsight>, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, category, readiness_score, readiness_state,
                    clinical_summary, goblin_reaction, actionable_tips
             FROM health_insights ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            Ok(HealthInsight {
                id: row.get(0)?,
                created_at: row.get(1)?,
                category: row.get(2)?,
                readiness_score: row.get(3)?,
                readiness_state: row.get(4)?,
                clinical_summary: row.get(5)?,
                goblin_reaction: row.get(6)?,
                actionable_tips: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut insights = Vec::new();
    for r in rows {
        insights.push(r.map_err(|e| e.to_string())?);
    }
    Ok(insights)
}

pub fn save_health_insight(
    db: Arc<Mutex<Connection>>,
    insight: &HealthInsight,
) -> Result<i64, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO health_insights (
            category, readiness_score, readiness_state, clinical_summary, goblin_reaction, actionable_tips
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &insight.category,
            insight.readiness_score,
            &insight.readiness_state,
            &insight.clinical_summary,
            &insight.goblin_reaction,
            &insight.actionable_tips,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

pub fn generate_deterministic_insight(bio: &GarminBiometrics) -> HealthInsight {
    let (readiness_score, readiness_state) = calculate_readiness(bio);

    let (clinical_summary, goblin_reaction, actionable_tips) = if readiness_score >= 80 {
        (
            format!(
                "Parasympathetic tone is elevated (HRV: {}ms, Sleep Score: {}). Overnight autonomic recovery is optimal, indicating high neuromuscular readiness and minimal physiological fatigue.",
                bio.hrv_rmssd, bio.sleep_score
            ),
            "Grah! Look at those numbers, mighty warrior! Your body battery is sizzling at 85%+! Time to crush a heavy dungeon crawl or smash a 10K sprint! Don't you dare slack today!".to_string(),
            "• Optimal day for high-intensity intervals or heavy compound lifting.\n• Consume adequate complex carbohydrates pre-workout.\n• Aim for 500+ active calories burned today.".to_string(),
        )
    } else if readiness_score >= 55 {
        (
            format!(
                "Biometrics demonstrate balanced homeostatic equilibrium (Body Battery: {}, Stress Index: {}). Cardiovascular metrics support moderate aerobic endurance and hypertrophy training.",
                bio.body_battery, bio.stress_level
            ),
            "Not bad, human! You are in decent shape today. We can raid a medium dungeon. Keep your hydration up and don't get lazy on the stairs!".to_string(),
            "• Moderate aerobic volume (Zone 2 cardio: 30-45 mins).\n• Focus on quality protein intake and hydration electrolytes.\n• Keep late afternoon caffeine intake minimal.".to_string(),
        )
    } else {
        (
            format!(
                "Sympathetic dominance detected (Resting HR elevated to {} bpm, Stress: {}, Sleep Score: {}). Cumulative physiological strain warrants immediate active recovery to prevent overtraining syndrome.",
                bio.resting_hr, bio.stress_level, bio.sleep_score
            ),
            "BY THE GOBLIN CHIEFTAIN! Your Body Battery is running on dungeon fumes! Put down the weights and drink a sleep potion before I have to drag you to your bed!".to_string(),
            "• Mandatory active recovery: light 20-min walking or mobility work only.\n• Avoid high-intensity stimulants and HIIT.\n• Target 8.5+ hours of sleep tonight with dark/cool room environment.".to_string(),
        )
    };

    HealthInsight {
        id: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        category: "Daily Readiness & Recovery".to_string(),
        readiness_score,
        readiness_state,
        clinical_summary,
        goblin_reaction,
        actionable_tips,
    }
}
