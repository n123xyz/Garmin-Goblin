use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbtThoughtRecord {
    pub id: i64,
    pub date: String,
    pub timestamp: String,
    pub situation: String,
    pub automatic_thought: String,
    pub distortions: Vec<String>,
    pub distress_before: i32,
    pub reframed_thought: String,
    pub distress_after: i32,
    pub garmin_stress: i32,
    pub garmin_hr: i32,
    pub goblin_advice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbtThoughtRecordInput {
    pub date: String,
    pub situation: String,
    pub automatic_thought: String,
    pub distortions: Vec<String>,
    pub distress_before: i32,
    pub reframed_thought: String,
    pub distress_after: i32,
    pub garmin_stress: Option<i32>,
    pub garmin_hr: Option<i32>,
    pub goblin_advice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbtReframeResult {
    pub reframed_thought: String,
    pub identified_distortions: Vec<String>,
    pub goblin_advice: String,
    pub suggested_somatic_action: String,
}

pub struct TherapyService;

impl TherapyService {
    pub fn save_cbt_record(
        conn: &Connection,
        input: CbtThoughtRecordInput,
    ) -> Result<CbtThoughtRecord, String> {
        let distortions_json = serde_json::to_string(&input.distortions)
            .unwrap_or_else(|_| "[]".to_string());

        let stress = input.garmin_stress.unwrap_or(0);
        let hr = input.garmin_hr.unwrap_or(0);

        conn.execute(
            "INSERT INTO cbt_thought_records (
                date, situation, automatic_thought, distortions_json, 
                distress_before, reframed_thought, distress_after, 
                garmin_stress, garmin_hr, goblin_advice
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &input.date,
                &input.situation,
                &input.automatic_thought,
                &distortions_json,
                input.distress_before,
                &input.reframed_thought,
                input.distress_after,
                stress,
                hr,
                &input.goblin_advice,
            ],
        )
        .map_err(|e| format!("Failed to save CBT thought record: {}", e))?;

        let id = conn.last_insert_rowid();

        let (timestamp,): (String,) = conn
            .query_row(
                "SELECT timestamp FROM cbt_thought_records WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap_or_else(|_| (input.date.clone(),));

        Ok(CbtThoughtRecord {
            id,
            date: input.date,
            timestamp,
            situation: input.situation,
            automatic_thought: input.automatic_thought,
            distortions: input.distortions,
            distress_before: input.distress_before,
            reframed_thought: input.reframed_thought,
            distress_after: input.distress_after,
            garmin_stress: stress,
            garmin_hr: hr,
            goblin_advice: input.goblin_advice,
        })
    }

    pub fn get_cbt_records(
        conn: &Connection,
        date: Option<String>,
        limit: usize,
    ) -> Result<Vec<CbtThoughtRecord>, String> {
        let mut query = "SELECT id, date, timestamp, situation, automatic_thought, distortions_json, 
                                distress_before, reframed_thought, distress_after, garmin_stress, 
                                garmin_hr, goblin_advice 
                         FROM cbt_thought_records ".to_string();

        if date.is_some() {
            query.push_str("WHERE date = ?1 ORDER BY id DESC LIMIT ?2");
        } else {
            query.push_str("ORDER BY id DESC LIMIT ?1");
        }

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let records = if let Some(ref d) = date {
            stmt.query_map(params![d, limit as i64], Self::map_row)
                .map_err(|e| e.to_string())?
        } else {
            stmt.query_map(params![limit as i64], Self::map_row)
                .map_err(|e| e.to_string())?
        };

        let mut list = Vec::new();
        for r in records {
            list.push(r.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    pub fn delete_cbt_record(conn: &Connection, id: i64) -> Result<bool, String> {
        let rows = conn
            .execute("DELETE FROM cbt_thought_records WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(rows > 0)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CbtThoughtRecord> {
        let distortions_raw: String = row.get(5)?;
        let distortions: Vec<String> =
            serde_json::from_str(&distortions_raw).unwrap_or_default();

        Ok(CbtThoughtRecord {
            id: row.get(0)?,
            date: row.get(1)?,
            timestamp: row.get(2)?,
            situation: row.get(3)?,
            automatic_thought: row.get(4)?,
            distortions,
            distress_before: row.get(6)?,
            reframed_thought: row.get(7)?,
            distress_after: row.get(8)?,
            garmin_stress: row.get(9)?,
            garmin_hr: row.get(10)?,
            goblin_advice: row.get(11)?,
        })
    }

    pub fn generate_deterministic_reframe(
        situation: &str,
        thought: &str,
        distortions: &[String],
        current_stress: i32,
        history_bio: &[crate::services::garmin::GarminBiometrics],
    ) -> CbtReframeResult {
        let distortion_str = if distortions.is_empty() {
            "negative self-criticism".to_string()
        } else {
            distortions.join(" and ")
        };

        let reframed = format!(
            "While it is natural to feel distressed about \"{}\", this thought (\"{}\") is heavily magnified by {}. A more grounded reality is: I am dealing with a challenging situation one moment at a time, my worth is not defined by perfection, and I have navigated difficult storms before.",
            situation.trim(),
            thought.trim(),
            distortion_str
        );

        let (days_n, avg_stress) = if !history_bio.is_empty() {
            let n = history_bio.len();
            let sum: i32 = history_bio.iter().map(|b| b.stress_level).sum();
            (n, sum / (n as i32))
        } else {
            (1, current_stress)
        };

        let goblin_advice = if avg_stress > 45 || current_stress > 50 {
            format!(
                "Listen closely, friend: looking across your last {} synced days, your average stress baseline has been elevated at {}/100. When cumulative nervous system fatigue builds up, the brain naturally interprets challenges as catastrophic emergencies. Drop your shoulders, unclamp your jaw, and give yourself grace—you are carrying biological strain.",
                days_n, avg_stress
            )
        } else {
            format!(
                "Even with a stable multi-day recovery baseline (avg stress {}/100), acute situations can shake the spirit. You caught the thought distortion—that is a massive victory. Take a slow, grounding breath.",
                avg_stress
            )
        };

        let somatic_action = if avg_stress > 45 || current_stress > 50 {
            "Perform 3 Physiological Sighs (two quick inhales through the nose, prolonged slow exhale through the mouth) to drop autonomic tone."
        } else {
            "Place a warm hand on your chest, take a slow 4-second belly breath, and notice your feet grounded on the earth."
        }
        .to_string();

        CbtReframeResult {
            reframed_thought: reframed,
            identified_distortions: distortions.to_vec(),
            goblin_advice,
            suggested_somatic_action: somatic_action,
        }
    }
}
