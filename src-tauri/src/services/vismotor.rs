use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VismotorSessionRecord {
    pub id: Option<i64>,
    pub date: String,
    pub timestamp: String,
    pub time_of_day: String,
    pub mode: String,
    pub total_trials: i64,
    pub correct_count: i64,
    pub incorrect_count: i64,
    pub accuracy: f64,
    pub mean_rt_ms: f64,
    pub mean_left_rt_ms: f64,
    pub mean_right_rt_ms: f64,
    pub hemispheric_difference_ms: f64,
    pub rt_std_dev_ms: f64,
    pub vismotor_score: i64,
    pub xp_awarded: i64,
    pub gold_awarded: i64,
    pub trial_data_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayVismotorSummary {
    pub has_completed_today: bool,
    pub latest_score: Option<i64>,
    pub latest_mean_rt_ms: Option<f64>,
    pub latest_accuracy: Option<f64>,
    pub sessions_count_today: i64,
    pub latest_session: Option<VismotorSessionRecord>,
}

pub struct VismotorService;

impl VismotorService {
    pub fn save_vismotor_session(
        conn: &Connection,
        mut session: VismotorSessionRecord,
    ) -> Result<VismotorSessionRecord, String> {
        conn.execute(
            "INSERT INTO vismotor_sessions (
                date, timestamp, time_of_day, mode, total_trials, correct_count,
                incorrect_count, accuracy, mean_rt_ms, mean_left_rt_ms, mean_right_rt_ms,
                hemispheric_difference_ms, rt_std_dev_ms, vismotor_score, xp_awarded,
                gold_awarded, trial_data_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                &session.date,
                &session.timestamp,
                &session.time_of_day,
                &session.mode,
                session.total_trials,
                session.correct_count,
                session.incorrect_count,
                session.accuracy,
                session.mean_rt_ms,
                session.mean_left_rt_ms,
                session.mean_right_rt_ms,
                session.hemispheric_difference_ms,
                session.rt_std_dev_ms,
                session.vismotor_score,
                session.xp_awarded,
                session.gold_awarded,
                &session.trial_data_json,
            ],
        )
        .map_err(|e| format!("Failed to insert VISMOTOR session: {}", e))?;

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

    pub fn get_vismotor_sessions_for_date(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<VismotorSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_trials, correct_count,
                        incorrect_count, accuracy, mean_rt_ms, mean_left_rt_ms, mean_right_rt_ms,
                        hemispheric_difference_ms, rt_std_dev_ms, vismotor_score, xp_awarded,
                        gold_awarded, trial_data_json
                 FROM vismotor_sessions
                 WHERE date = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([date], |row| {
                Ok(VismotorSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_trials: row.get(5)?,
                    correct_count: row.get(6)?,
                    incorrect_count: row.get(7)?,
                    accuracy: row.get(8)?,
                    mean_rt_ms: row.get(9)?,
                    mean_left_rt_ms: row.get(10)?,
                    mean_right_rt_ms: row.get(11)?,
                    hemispheric_difference_ms: row.get(12)?,
                    rt_std_dev_ms: row.get(13)?,
                    vismotor_score: row.get(14)?,
                    xp_awarded: row.get(15)?,
                    gold_awarded: row.get(16)?,
                    trial_data_json: row.get(17)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_recent_vismotor_sessions(
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<VismotorSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_trials, correct_count,
                        incorrect_count, accuracy, mean_rt_ms, mean_left_rt_ms, mean_right_rt_ms,
                        hemispheric_difference_ms, rt_std_dev_ms, vismotor_score, xp_awarded,
                        gold_awarded, trial_data_json
                 FROM vismotor_sessions
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(VismotorSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_trials: row.get(5)?,
                    correct_count: row.get(6)?,
                    incorrect_count: row.get(7)?,
                    accuracy: row.get(8)?,
                    mean_rt_ms: row.get(9)?,
                    mean_left_rt_ms: row.get(10)?,
                    mean_right_rt_ms: row.get(11)?,
                    hemispheric_difference_ms: row.get(12)?,
                    rt_std_dev_ms: row.get(13)?,
                    vismotor_score: row.get(14)?,
                    xp_awarded: row.get(15)?,
                    gold_awarded: row.get(16)?,
                    trial_data_json: row.get(17)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_today_vismotor_summary(
        conn: &Connection,
        today_date: &str,
    ) -> Result<TodayVismotorSummary, String> {
        let sessions = Self::get_vismotor_sessions_for_date(conn, today_date)?;
        let count = sessions.len() as i64;

        if let Some(latest) = sessions.last() {
            Ok(TodayVismotorSummary {
                has_completed_today: true,
                latest_score: Some(latest.vismotor_score),
                latest_mean_rt_ms: Some(latest.mean_rt_ms),
                latest_accuracy: Some(latest.accuracy),
                sessions_count_today: count,
                latest_session: Some(latest.clone()),
            })
        } else {
            Ok(TodayVismotorSummary {
                has_completed_today: false,
                latest_score: None,
                latest_mean_rt_ms: None,
                latest_accuracy: None,
                sessions_count_today: 0,
                latest_session: None,
            })
        }
    }
}
