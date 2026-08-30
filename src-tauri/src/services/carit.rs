use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaritSessionRecord {
    pub id: Option<i64>,
    pub date: String,
    pub timestamp: String,
    pub time_of_day: String,
    pub mode: String,
    pub total_trials: i64,
    pub hit_count: i64,
    pub miss_count: i64,
    pub false_alarm_count: i64,
    pub corr_reject_count: i64,
    pub total_accuracy: f64,
    pub go_accuracy: f64,
    pub nogo_accuracy: f64,
    pub mean_rt_ms: f64,
    pub median_rt_ms: f64,
    pub rt_std_dev_ms: f64,
    pub d_prime: f64,
    pub cognitive_score: i64,
    pub xp_awarded: i64,
    pub gold_awarded: i64,
    pub trial_data_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayCaritSummary {
    pub has_completed_today: bool,
    pub latest_score: Option<i64>,
    pub latest_mean_rt_ms: Option<f64>,
    pub latest_inhibition_acc: Option<f64>,
    pub sessions_count_today: i64,
    pub latest_session: Option<CaritSessionRecord>,
}

pub struct CaritService;

impl CaritService {
    pub fn save_carit_session(
        conn: &Connection,
        mut session: CaritSessionRecord,
    ) -> Result<CaritSessionRecord, String> {
        conn.execute(
            "INSERT INTO carit_sessions (
                date, timestamp, time_of_day, mode, total_trials, hit_count, miss_count,
                false_alarm_count, corr_reject_count, total_accuracy, go_accuracy, nogo_accuracy,
                mean_rt_ms, median_rt_ms, rt_std_dev_ms, d_prime, cognitive_score, xp_awarded,
                gold_awarded, trial_data_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            rusqlite::params![
                &session.date,
                &session.timestamp,
                &session.time_of_day,
                &session.mode,
                session.total_trials,
                session.hit_count,
                session.miss_count,
                session.false_alarm_count,
                session.corr_reject_count,
                session.total_accuracy,
                session.go_accuracy,
                session.nogo_accuracy,
                session.mean_rt_ms,
                session.median_rt_ms,
                session.rt_std_dev_ms,
                session.d_prime,
                session.cognitive_score,
                session.xp_awarded,
                session.gold_awarded,
                &session.trial_data_json,
            ],
        )
        .map_err(|e| format!("Failed to insert CARIT session: {}", e))?;

        let row_id = conn.last_insert_rowid();
        session.id = Some(row_id);

        // Award Goblin XP and Gold for the cognitive training session
        if session.xp_awarded > 0 || session.gold_awarded > 0 {
            let _ = crate::services::goblin::award_xp_and_gold(
                conn,
                session.xp_awarded as i32,
                session.gold_awarded as i32,
            );
        }

        Ok(session)
    }

    pub fn get_carit_sessions_for_date(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<CaritSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_trials, hit_count, miss_count,
                        false_alarm_count, corr_reject_count, total_accuracy, go_accuracy, nogo_accuracy,
                        mean_rt_ms, median_rt_ms, rt_std_dev_ms, d_prime, cognitive_score, xp_awarded,
                        gold_awarded, trial_data_json
                 FROM carit_sessions
                 WHERE date = ?1
                 ORDER BY timestamp ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([date], |row| {
                Ok(CaritSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_trials: row.get(5)?,
                    hit_count: row.get(6)?,
                    miss_count: row.get(7)?,
                    false_alarm_count: row.get(8)?,
                    corr_reject_count: row.get(9)?,
                    total_accuracy: row.get(10)?,
                    go_accuracy: row.get(11)?,
                    nogo_accuracy: row.get(12)?,
                    mean_rt_ms: row.get(13)?,
                    median_rt_ms: row.get(14)?,
                    rt_std_dev_ms: row.get(15)?,
                    d_prime: row.get(16)?,
                    cognitive_score: row.get(17)?,
                    xp_awarded: row.get(18)?,
                    gold_awarded: row.get(19)?,
                    trial_data_json: row.get(20)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_recent_carit_sessions(
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<CaritSessionRecord>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, timestamp, time_of_day, mode, total_trials, hit_count, miss_count,
                        false_alarm_count, corr_reject_count, total_accuracy, go_accuracy, nogo_accuracy,
                        mean_rt_ms, median_rt_ms, rt_std_dev_ms, d_prime, cognitive_score, xp_awarded,
                        gold_awarded, trial_data_json
                 FROM carit_sessions
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(CaritSessionRecord {
                    id: Some(row.get(0)?),
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3)?,
                    mode: row.get(4)?,
                    total_trials: row.get(5)?,
                    hit_count: row.get(6)?,
                    miss_count: row.get(7)?,
                    false_alarm_count: row.get(8)?,
                    corr_reject_count: row.get(9)?,
                    total_accuracy: row.get(10)?,
                    go_accuracy: row.get(11)?,
                    nogo_accuracy: row.get(12)?,
                    mean_rt_ms: row.get(13)?,
                    median_rt_ms: row.get(14)?,
                    rt_std_dev_ms: row.get(15)?,
                    d_prime: row.get(16)?,
                    cognitive_score: row.get(17)?,
                    xp_awarded: row.get(18)?,
                    gold_awarded: row.get(19)?,
                    trial_data_json: row.get(20)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for sess in rows.flatten() {
            results.push(sess);
        }
        Ok(results)
    }

    pub fn get_today_carit_summary(
        conn: &Connection,
        today_date: &str,
    ) -> Result<TodayCaritSummary, String> {
        let sessions = Self::get_carit_sessions_for_date(conn, today_date)?;
        let count = sessions.len() as i64;

        if let Some(latest) = sessions.last() {
            Ok(TodayCaritSummary {
                has_completed_today: true,
                latest_score: Some(latest.cognitive_score),
                latest_mean_rt_ms: Some(latest.mean_rt_ms),
                latest_inhibition_acc: Some(latest.nogo_accuracy),
                sessions_count_today: count,
                latest_session: Some(latest.clone()),
            })
        } else {
            Ok(TodayCaritSummary {
                has_completed_today: false,
                latest_score: None,
                latest_mean_rt_ms: None,
                latest_inhibition_acc: None,
                sessions_count_today: 0,
                latest_session: None,
            })
        }
    }
}
