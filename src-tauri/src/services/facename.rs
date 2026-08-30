use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceNameSessionRecord {
    pub id: Option<i64>,
    pub date: String,
    pub timestamp: String,
    pub time_of_day: String,
    pub mode: String,
    pub total_memorized: i64,
    pub total_recalled: i64,
    pub correct_recall_count: i64,
    pub recall_accuracy: f64,
    pub mean_recall_rt_ms: f64,
    pub median_recall_rt_ms: f64,
    pub distractor_accuracy: f64,
    pub memory_score: i64,
    pub xp_awarded: i64,
    pub gold_awarded: i64,
    pub trial_data_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayFaceNameSummary {
    pub has_completed_today: bool,
    pub latest_score: Option<i64>,
    pub latest_recall_accuracy: Option<f64>,
    pub latest_recall_rt_ms: Option<f64>,
    pub sessions_count_today: i64,
    pub latest_session: Option<FaceNameSessionRecord>,
}

pub struct FaceNameService;

impl FaceNameService {
    pub fn save_facename_session(
        conn: &Connection,
        mut session: FaceNameSessionRecord,
    ) -> Result<FaceNameSessionRecord, String> {
        conn.execute(
            "INSERT INTO facename_sessions (
                date, timestamp, time_of_day, mode, total_memorized, total_recalled,
                correct_recall_count, recall_accuracy, mean_recall_rt_ms, median_recall_rt_ms,
                distractor_accuracy, memory_score, xp_awarded, gold_awarded, trial_data_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                &session.date,
                &session.timestamp,
                &session.time_of_day,
                &session.mode,
                session.total_memorized,
                session.total_recalled,
                session.correct_recall_count,
                session.recall_accuracy,
                session.mean_recall_rt_ms,
                session.median_recall_rt_ms,
                session.distractor_accuracy,
                session.memory_score,
                session.xp_awarded,
                session.gold_awarded,
                &session.trial_data_json,
            ],
        )
        .map_err(|e| format!("Failed to insert FACENAME session: {}", e))?;

        let row_id = conn.last_insert_rowid();
        session.id = Some(row_id);

        if session.xp_awarded > 0 || session.gold_awarded > 0 {
            let _ = crate::services::goblin::award_xp_and_gold(
                conn,
                session.xp_awarded as i32,
                session.gold_awarded as i32,
            );
        }

        Ok(session)
    }

    pub fn get_facename_sessions_for_date(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<FaceNameSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_memorized, total_recalled,
                        correct_recall_count, recall_accuracy, mean_recall_rt_ms, median_recall_rt_ms,
                        distractor_accuracy, memory_score, xp_awarded, gold_awarded, trial_data_json
                 FROM facename_sessions
                 WHERE date = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([date], |row| {
                Ok(FaceNameSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_memorized: row.get(5)?,
                    total_recalled: row.get(6)?,
                    correct_recall_count: row.get(7)?,
                    recall_accuracy: row.get(8)?,
                    mean_recall_rt_ms: row.get(9)?,
                    median_recall_rt_ms: row.get(10)?,
                    distractor_accuracy: row.get(11)?,
                    memory_score: row.get(12)?,
                    xp_awarded: row.get(13)?,
                    gold_awarded: row.get(14)?,
                    trial_data_json: row.get(15)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_recent_facename_sessions(
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<FaceNameSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_memorized, total_recalled,
                        correct_recall_count, recall_accuracy, mean_recall_rt_ms, median_recall_rt_ms,
                        distractor_accuracy, memory_score, xp_awarded, gold_awarded, trial_data_json
                 FROM facename_sessions
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(FaceNameSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_memorized: row.get(5)?,
                    total_recalled: row.get(6)?,
                    correct_recall_count: row.get(7)?,
                    recall_accuracy: row.get(8)?,
                    mean_recall_rt_ms: row.get(9)?,
                    median_recall_rt_ms: row.get(10)?,
                    distractor_accuracy: row.get(11)?,
                    memory_score: row.get(12)?,
                    xp_awarded: row.get(13)?,
                    gold_awarded: row.get(14)?,
                    trial_data_json: row.get(15)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_today_facename_summary(
        conn: &Connection,
        today_date: &str,
    ) -> Result<TodayFaceNameSummary, String> {
        let sessions = Self::get_facename_sessions_for_date(conn, today_date)?;
        let count = sessions.len() as i64;

        if let Some(latest) = sessions.last() {
            Ok(TodayFaceNameSummary {
                has_completed_today: true,
                latest_score: Some(latest.memory_score),
                latest_recall_accuracy: Some(latest.recall_accuracy),
                latest_recall_rt_ms: Some(latest.mean_recall_rt_ms),
                sessions_count_today: count,
                latest_session: Some(latest.clone()),
            })
        } else {
            Ok(TodayFaceNameSummary {
                has_completed_today: false,
                latest_score: None,
                latest_recall_accuracy: None,
                latest_recall_rt_ms: None,
                sessions_count_today: 0,
                latest_session: None,
            })
        }
    }
}
