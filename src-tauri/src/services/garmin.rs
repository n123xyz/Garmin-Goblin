use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GarminBiometrics {
    pub id: i64,
    pub timestamp: String,
    pub date: String,
    pub body_battery: i32,
    pub body_battery_drain: i32,
    pub body_battery_charge: i32,
    pub stress_level: i32,
    pub resting_hr: i32,
    pub current_hr: i32,
    pub hrv_status: String,
    pub hrv_rmssd: i32,
    pub sleep_score: i32,
    pub sleep_duration_sec: i32,
    pub sleep_deep_sec: i32,
    pub sleep_rem_sec: i32,
    pub sleep_light_sec: i32,
    pub sleep_awake_sec: i32,
    pub steps: i32,
    pub step_goal: i32,
    pub active_calories: i32,
    pub total_calories: i32,
    pub intensity_minutes: i32,
    pub vo2_max: f64,
    pub spo2: i32,
    pub respiration_rate: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GarminActivity {
    pub id: i64,
    pub activity_type: String,
    pub title: String,
    pub start_time: String,
    pub duration_sec: i32,
    pub distance_meters: f64,
    pub calories: i32,
    pub avg_hr: i32,
    pub max_hr: i32,
    pub aerobic_training_effect: f64,
    pub anaerobic_training_effect: f64,
}

pub fn get_latest_biometrics(db: Arc<Mutex<Connection>>) -> Result<GarminBiometrics, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM garmin_biometrics", [], |r| r.get(0))
        .unwrap_or(0);

    if count == 0 {
        let _ = conn.execute(
            "INSERT INTO garmin_biometrics (
                date, body_battery, body_battery_drain, body_battery_charge,
                stress_level, resting_hr, current_hr, hrv_status, hrv_rmssd,
                sleep_score, sleep_duration_sec, sleep_deep_sec, sleep_rem_sec, sleep_light_sec, sleep_awake_sec,
                steps, step_goal, active_calories, total_calories, intensity_minutes, vo2_max, spo2, respiration_rate
            ) VALUES (
                date('now', 'localtime'), 0, 0, 0,
                0, 0, 0, 'No Data', 0,
                0, 0, 0, 0, 0, 0,
                0, 10000, 0, 0, 0, 0.0, 0, 0
            )",
            [],
        );
    }

    let map_row = |row: &rusqlite::Row| {
        Ok(GarminBiometrics {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            date: row.get(2)?,
            body_battery: row.get(3)?,
            body_battery_drain: row.get(4)?,
            body_battery_charge: row.get(5)?,
            stress_level: row.get(6)?,
            resting_hr: row.get(7)?,
            current_hr: row.get(8)?,
            hrv_status: row.get(9)?,
            hrv_rmssd: row.get(10)?,
            sleep_score: row.get(11)?,
            sleep_duration_sec: row.get(12)?,
            sleep_deep_sec: row.get(13)?,
            sleep_rem_sec: row.get(14)?,
            sleep_light_sec: row.get(15)?,
            sleep_awake_sec: row.get(16)?,
            steps: row.get(17)?,
            step_goal: row.get(18)?,
            active_calories: row.get(19)?,
            total_calories: row.get(20)?,
            intensity_minutes: row.get(21)?,
            vo2_max: row.get(22)?,
            spo2: row.get(23)?,
            respiration_rate: row.get(24)?,
        })
    };

    let sql = "SELECT 
        COALESCE((SELECT id FROM garmin_biometrics WHERE date <= date('now', 'localtime') AND (steps > 0 OR resting_hr > 0 OR sleep_score > 0 OR hrv_rmssd > 0 OR body_battery > 0) ORDER BY date DESC, id DESC LIMIT 1), 1),
        COALESCE((SELECT timestamp FROM garmin_biometrics WHERE date <= date('now', 'localtime') AND (steps > 0 OR resting_hr > 0 OR sleep_score > 0 OR hrv_rmssd > 0 OR body_battery > 0) ORDER BY date DESC, id DESC LIMIT 1), CURRENT_TIMESTAMP),
        COALESCE((SELECT date FROM garmin_biometrics WHERE date <= date('now', 'localtime') AND (steps > 0 OR resting_hr > 0 OR sleep_score > 0 OR hrv_rmssd > 0 OR body_battery > 0) ORDER BY date DESC, id DESC LIMIT 1), date('now', 'localtime')),
        COALESCE((SELECT body_battery FROM garmin_biometrics WHERE body_battery > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT body_battery_drain FROM garmin_biometrics WHERE body_battery_drain > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT body_battery_charge FROM garmin_biometrics WHERE body_battery_charge > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT stress_level FROM garmin_biometrics WHERE stress_level > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT resting_hr FROM garmin_biometrics WHERE resting_hr > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT current_hr FROM garmin_biometrics WHERE current_hr > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT hrv_status FROM garmin_biometrics WHERE hrv_status != '' AND hrv_status != 'No Data' AND hrv_status != 'none' ORDER BY date DESC, id DESC LIMIT 1), 'Balanced'),
        COALESCE((SELECT hrv_rmssd FROM garmin_biometrics WHERE hrv_rmssd > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_score FROM garmin_biometrics WHERE sleep_score > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_duration_sec FROM garmin_biometrics WHERE sleep_duration_sec > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_deep_sec FROM garmin_biometrics WHERE sleep_deep_sec > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_rem_sec FROM garmin_biometrics WHERE sleep_rem_sec > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_light_sec FROM garmin_biometrics WHERE sleep_light_sec > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT sleep_awake_sec FROM garmin_biometrics WHERE sleep_awake_sec > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT steps FROM garmin_biometrics WHERE date <= date('now', 'localtime') AND steps > 0 ORDER BY date DESC, steps DESC, id DESC LIMIT 1), 0),
        10000,
        COALESCE((SELECT active_calories FROM garmin_biometrics WHERE active_calories > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT total_calories FROM garmin_biometrics WHERE total_calories > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT intensity_minutes FROM garmin_biometrics WHERE intensity_minutes > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT vo2_max FROM garmin_biometrics WHERE vo2_max > 0 ORDER BY date DESC, id DESC LIMIT 1), 0.0),
        COALESCE((SELECT spo2 FROM garmin_biometrics WHERE spo2 > 0 ORDER BY date DESC, id DESC LIMIT 1), 0),
        COALESCE((SELECT respiration_rate FROM garmin_biometrics WHERE respiration_rate > 0 ORDER BY date DESC, id DESC LIMIT 1), 0)";

    let bio = conn.query_row(sql, [], map_row).map_err(|e| e.to_string())?;

    eprintln!("[GarminGoblin DB] get_latest_biometrics returning: id={}, date={}, steps={}, hr={}, rhr={}, bb={}, stress={}, hrv='{}' (rmssd={}), sleep={}",
        bio.id, bio.date, bio.steps, bio.current_hr, bio.resting_hr, bio.body_battery, bio.stress_level, bio.hrv_status, bio.hrv_rmssd, bio.sleep_score);

    Ok(bio)
}

pub fn get_biometrics_history(
    db: Arc<Mutex<Connection>>,
    limit: usize,
) -> Result<Vec<GarminBiometrics>, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, date, body_battery, body_battery_drain, body_battery_charge,
                    stress_level, resting_hr, current_hr, hrv_status, hrv_rmssd,
                    sleep_score, sleep_duration_sec, sleep_deep_sec, sleep_rem_sec, sleep_light_sec, sleep_awake_sec,
                    steps, step_goal, active_calories, total_calories, intensity_minutes, vo2_max, spo2, respiration_rate
             FROM garmin_biometrics 
             ORDER BY date DESC, id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            Ok(GarminBiometrics {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                date: row.get(2)?,
                body_battery: row.get(3)?,
                body_battery_drain: row.get(4)?,
                body_battery_charge: row.get(5)?,
                stress_level: row.get(6)?,
                resting_hr: row.get(7)?,
                current_hr: row.get(8)?,
                hrv_status: row.get(9)?,
                hrv_rmssd: row.get(10)?,
                sleep_score: row.get(11)?,
                sleep_duration_sec: row.get(12)?,
                sleep_deep_sec: row.get(13)?,
                sleep_rem_sec: row.get(14)?,
                sleep_light_sec: row.get(15)?,
                sleep_awake_sec: row.get(16)?,
                steps: row.get(17)?,
                step_goal: row.get(18)?,
                active_calories: row.get(19)?,
                total_calories: row.get(20)?,
                intensity_minutes: row.get(21)?,
                vo2_max: row.get(22)?,
                spo2: row.get(23)?,
                respiration_rate: row.get(24)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r.map_err(|e| e.to_string())?);
    }
    result.reverse();
    Ok(result)
}

pub fn update_biometrics(
    db: Arc<Mutex<Connection>>,
    bio: GarminBiometrics,
) -> Result<GarminBiometrics, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO garmin_biometrics (
            date, body_battery, body_battery_drain, body_battery_charge,
            stress_level, resting_hr, current_hr, hrv_status, hrv_rmssd,
            sleep_score, sleep_duration_sec, sleep_deep_sec, sleep_rem_sec, sleep_light_sec, sleep_awake_sec,
            steps, step_goal, active_calories, total_calories, intensity_minutes, vo2_max, spo2, respiration_rate
        ) VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
        )",
        rusqlite::params![
            &bio.date,
            bio.body_battery,
            bio.body_battery_drain,
            bio.body_battery_charge,
            bio.stress_level,
            bio.resting_hr,
            bio.current_hr,
            &bio.hrv_status,
            bio.hrv_rmssd,
            bio.sleep_score,
            bio.sleep_duration_sec,
            bio.sleep_deep_sec,
            bio.sleep_rem_sec,
            bio.sleep_light_sec,
            bio.sleep_awake_sec,
            bio.steps,
            bio.step_goal,
            bio.active_calories,
            bio.total_calories,
            bio.intensity_minutes,
            bio.vo2_max,
            bio.spo2,
            bio.respiration_rate,
        ],
    )
    .map_err(|e| e.to_string())?;

    let new_id = conn.last_insert_rowid();
    let mut updated = bio;
    updated.id = new_id;

    // Also trigger quest progress update based on new biometrics
    drop(conn);
    let _ = crate::services::goblin::update_quest_progress(db.clone(), &updated);
    let _ = crate::services::goblin::update_goblin_mood_from_biometrics(db.clone(), &updated);

    Ok(updated)
}

pub fn get_activities(
    db: Arc<Mutex<Connection>>,
    limit: usize,
) -> Result<Vec<GarminActivity>, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, activity_type, title, start_time, duration_sec, distance_meters,
                    calories, avg_hr, max_hr, aerobic_training_effect, anaerobic_training_effect
             FROM garmin_activities
             ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            Ok(GarminActivity {
                id: row.get(0)?,
                activity_type: row.get(1)?,
                title: row.get(2)?,
                start_time: row.get(3)?,
                duration_sec: row.get(4)?,
                distance_meters: row.get(5)?,
                calories: row.get(6)?,
                avg_hr: row.get(7)?,
                max_hr: row.get(8)?,
                aerobic_training_effect: row.get(9)?,
                anaerobic_training_effect: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

pub fn log_activity(
    db: Arc<Mutex<Connection>>,
    activity: GarminActivity,
) -> Result<GarminActivity, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "INSERT INTO garmin_activities (
            activity_type, title, start_time, duration_sec, distance_meters,
            calories, avg_hr, max_hr, aerobic_training_effect, anaerobic_training_effect
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &activity.activity_type,
            &activity.title,
            &activity.start_time,
            activity.duration_sec,
            activity.distance_meters,
            activity.calories,
            activity.avg_hr,
            activity.max_hr,
            activity.aerobic_training_effect,
            activity.anaerobic_training_effect,
        ),
    )
    .map_err(|e| e.to_string())?;

    let new_id = conn.last_insert_rowid();
    let mut updated = activity;
    updated.id = new_id;

    // Add Goblin XP and Gold for workout!
    drop(conn);
    let xp_reward = ((updated.duration_sec as f64 / 60.0) * 4.0) as i32 + 50;
    let gold_reward = ((updated.calories as f64) * 0.05) as i32 + 10;
    let _ = crate::services::goblin::add_xp_and_gold(db.clone(), xp_reward, gold_reward);

    Ok(updated)
}
