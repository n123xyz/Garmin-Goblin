use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodLog {
    pub id: i64,
    pub date: String,
    pub timestamp: String,
    pub meal_type: String,
    pub description: String,
    pub calories: i32,
    pub carbs_g: f64,
    pub protein_g: f64,
    pub fat_g: f64,
    pub fiber_g: f64,
    pub sugar_g: f64,
    pub sodium_mg: f64,
    pub water_ml: i32,
    pub notes: Option<String>,
    pub image_path: Option<String>,
    pub source_type: String,
    pub confidence_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodLogInput {
    pub id: Option<i64>,
    pub date: String,
    pub meal_type: String,
    pub description: String,
    pub calories: Option<i32>,
    pub carbs_g: Option<f64>,
    pub protein_g: Option<f64>,
    pub fat_g: Option<f64>,
    pub fiber_g: Option<f64>,
    pub sugar_g: Option<f64>,
    pub sodium_mg: Option<f64>,
    pub water_ml: Option<i32>,
    pub notes: Option<String>,
    pub image_path: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyNutritionSummary {
    pub date: String,
    pub total_calories: i32,
    pub total_carbs_g: f64,
    pub total_protein_g: f64,
    pub total_fat_g: f64,
    pub total_water_ml: i32,
    pub meal_count: usize,
    pub logs: Vec<FoodLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmotionalLog {
    pub id: i64,
    pub date: String,
    pub timestamp: String,
    pub time_of_day: String,
    pub mood: String,
    pub energy_level: i32,
    pub motivation_level: i32,
    pub stress_level: i32,
    pub perceived_recovery: i32,
    pub stressors: Vec<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmotionalLogInput {
    pub id: Option<i64>,
    pub date: String,
    pub time_of_day: Option<String>,
    pub mood: String,
    pub energy_level: i32,
    pub motivation_level: i32,
    pub stress_level: i32,
    pub perceived_recovery: i32,
    pub stressors: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarActivityPill {
    pub id: i64,
    pub sport: String,
    pub title: String,
    pub duration_min: i32,
    pub distance_km: f64,
    pub calories: i32,
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDaySummary {
    pub date: String,
    pub day_of_month: u32,
    pub is_current_month: bool,
    pub is_today: bool,
    // Biometrics
    pub sleep_score: i32,
    pub steps: i32,
    pub resting_hr: i32,
    pub hrv_status: String,
    pub hrv_rmssd: i32,
    pub stress_level: i32,
    // Aggregates
    pub total_food_calories: i32,
    pub total_active_calories: i32,
    pub food_count: usize,
    pub activity_count: usize,
    pub latest_mood: Option<String>,
    pub avg_emotional_stress: Option<f64>,
    pub avg_energy_level: Option<f64>,
    // Detailed items
    pub activities: Vec<CalendarActivityPill>,
    pub foods: Vec<FoodLog>,
    pub emotions: Vec<EmotionalLog>,
    pub events: Vec<crate::services::calendar::CalendarEventItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarMonthResponse {
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub days: Vec<CalendarDaySummary>,
}

pub struct JournalService;

impl JournalService {
    // --- Food Logs ---
    pub fn get_food_logs(
        conn: &Connection,
        date_str: Option<String>,
        limit: Option<usize>,
    ) -> SqlResult<Vec<FoodLog>> {
        let max_rows = limit.unwrap_or(50);
        let mut query = "SELECT id, date, timestamp, meal_type, description, calories, carbs_g, protein_g, fat_g, fiber_g, sugar_g, sodium_mg, water_ml, notes, image_path, source_type, confidence_score FROM food_logs".to_string();

        if let Some(target_date) = date_str {
            query.push_str(" WHERE date = ?1 ORDER BY timestamp DESC, id DESC LIMIT ?2");
            let mut stmt = conn.prepare(&query)?;
            let rows = stmt.query_map(params![target_date, max_rows as i64], |row| {
                Ok(FoodLog {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    meal_type: row.get(3)?,
                    description: row.get(4)?,
                    calories: row.get(5)?,
                    carbs_g: row.get(6)?,
                    protein_g: row.get(7)?,
                    fat_g: row.get(8)?,
                    fiber_g: row.get(9)?,
                    sugar_g: row.get(10)?,
                    sodium_mg: row.get(11)?,
                    water_ml: row.get(12)?,
                    notes: row.get(13)?,
                    image_path: row.get(14)?,
                    source_type: row.get(15)?,
                    confidence_score: row.get(16)?,
                })
            })?;
            rows.collect()
        } else {
            query.push_str(" ORDER BY date DESC, timestamp DESC, id DESC LIMIT ?1");
            let mut stmt = conn.prepare(&query)?;
            let rows = stmt.query_map(params![max_rows as i64], |row| {
                Ok(FoodLog {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    meal_type: row.get(3)?,
                    description: row.get(4)?,
                    calories: row.get(5)?,
                    carbs_g: row.get(6)?,
                    protein_g: row.get(7)?,
                    fat_g: row.get(8)?,
                    fiber_g: row.get(9)?,
                    sugar_g: row.get(10)?,
                    sodium_mg: row.get(11)?,
                    water_ml: row.get(12)?,
                    notes: row.get(13)?,
                    image_path: row.get(14)?,
                    source_type: row.get(15)?,
                    confidence_score: row.get(16)?,
                })
            })?;
            rows.collect()
        }
    }

    pub fn save_food_log(conn: &Connection, input: FoodLogInput) -> SqlResult<FoodLog> {
        let calories = input.calories.unwrap_or(0);
        let carbs = input.carbs_g.unwrap_or(0.0);
        let protein = input.protein_g.unwrap_or(0.0);
        let fat = input.fat_g.unwrap_or(0.0);
        let fiber = input.fiber_g.unwrap_or(0.0);
        let sugar = input.sugar_g.unwrap_or(0.0);
        let sodium = input.sodium_mg.unwrap_or(0.0);
        let water = input.water_ml.unwrap_or(0);
        let source_type = input.source_type.unwrap_or_else(|| "manual".to_string());

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE food_logs SET 
                    date = ?1, meal_type = ?2, description = ?3, calories = ?4, 
                    carbs_g = ?5, protein_g = ?6, fat_g = ?7, fiber_g = ?8, 
                    sugar_g = ?9, sodium_mg = ?10, water_ml = ?11, notes = ?12, 
                    image_path = ?13, source_type = ?14 
                WHERE id = ?15",
                params![
                    input.date,
                    input.meal_type,
                    input.description,
                    calories,
                    carbs,
                    protein,
                    fat,
                    fiber,
                    sugar,
                    sodium,
                    water,
                    input.notes,
                    input.image_path,
                    source_type,
                    id
                ],
            )?;
            let logs = Self::get_food_logs(conn, Some(input.date.clone()), Some(100))?;
            Ok(logs.into_iter().find(|l| l.id == id).unwrap())
        } else {
            conn.execute(
                "INSERT INTO food_logs (
                    date, meal_type, description, calories, carbs_g, protein_g, 
                    fat_g, fiber_g, sugar_g, sodium_mg, water_ml, notes, image_path, source_type
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    input.date,
                    input.meal_type,
                    input.description,
                    calories,
                    carbs,
                    protein,
                    fat,
                    fiber,
                    sugar,
                    sodium,
                    water,
                    input.notes,
                    input.image_path,
                    source_type,
                ],
            )?;
            let last_id = conn.last_insert_rowid();
            let logs = Self::get_food_logs(conn, Some(input.date.clone()), Some(100))?;
            Ok(logs.into_iter().find(|l| l.id == last_id).unwrap())
        }
    }

    pub fn delete_food_log(conn: &Connection, id: i64) -> SqlResult<bool> {
        let count = conn.execute("DELETE FROM food_logs WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    pub fn get_daily_nutrition_summary(conn: &Connection, date_str: String) -> SqlResult<DailyNutritionSummary> {
        let logs = Self::get_food_logs(conn, Some(date_str.clone()), Some(100))?;
        let mut total_calories = 0;
        let mut total_carbs = 0.0;
        let mut total_protein = 0.0;
        let mut total_fat = 0.0;
        let mut total_water = 0;

        for log in &logs {
            total_calories += log.calories;
            total_carbs += log.carbs_g;
            total_protein += log.protein_g;
            total_fat += log.fat_g;
            total_water += log.water_ml;
        }

        Ok(DailyNutritionSummary {
            date: date_str,
            total_calories,
            total_carbs_g: (total_carbs * 10.0).round() / 10.0,
            total_protein_g: (total_protein * 10.0).round() / 10.0,
            total_fat_g: (total_fat * 10.0).round() / 10.0,
            total_water_ml: total_water,
            meal_count: logs.len(),
            logs,
        })
    }

    // --- Emotional Logs ---
    pub fn get_emotional_logs(
        conn: &Connection,
        date_str: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> SqlResult<Vec<EmotionalLog>> {
        let mut query = "SELECT id, date, timestamp, time_of_day, mood, energy_level, motivation_level, stress_level, perceived_recovery, stressors_json, notes, created_at FROM emotional_logs".to_string();

        if let Some(date) = date_str {
            query.push_str(" WHERE date = ?1 ORDER BY timestamp DESC, id DESC");
            let mut stmt = conn.prepare(&query)?;
            let rows = stmt.query_map(params![date], |row| {
                let stressors_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
                let stressors: Vec<String> = serde_json::from_str(&stressors_json).unwrap_or_default();
                Ok(EmotionalLog {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3).unwrap_or_else(|_| "afternoon".to_string()),
                    mood: row.get(4)?,
                    energy_level: row.get(5)?,
                    motivation_level: row.get(6)?,
                    stress_level: row.get(7)?,
                    perceived_recovery: row.get(8)?,
                    stressors,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?;
            rows.collect()
        } else if let (Some(start), Some(end)) = (start_date, end_date) {
            query.push_str(" WHERE date >= ?1 AND date <= ?2 ORDER BY date DESC, timestamp DESC, id DESC");
            let mut stmt = conn.prepare(&query)?;
            let rows = stmt.query_map(params![start, end], |row| {
                let stressors_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
                let stressors: Vec<String> = serde_json::from_str(&stressors_json).unwrap_or_default();
                Ok(EmotionalLog {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3).unwrap_or_else(|_| "afternoon".to_string()),
                    mood: row.get(4)?,
                    energy_level: row.get(5)?,
                    motivation_level: row.get(6)?,
                    stress_level: row.get(7)?,
                    perceived_recovery: row.get(8)?,
                    stressors,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?;
            rows.collect()
        } else {
            query.push_str(" ORDER BY date DESC, timestamp DESC, id DESC LIMIT 100");
            let mut stmt = conn.prepare(&query)?;
            let rows = stmt.query_map([], |row| {
                let stressors_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
                let stressors: Vec<String> = serde_json::from_str(&stressors_json).unwrap_or_default();
                Ok(EmotionalLog {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    timestamp: row.get(2)?,
                    time_of_day: row.get(3).unwrap_or_else(|_| "afternoon".to_string()),
                    mood: row.get(4)?,
                    energy_level: row.get(5)?,
                    motivation_level: row.get(6)?,
                    stress_level: row.get(7)?,
                    perceived_recovery: row.get(8)?,
                    stressors,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?;
            rows.collect()
        }
    }

    pub fn save_emotional_log(conn: &Connection, input: EmotionalLogInput) -> SqlResult<EmotionalLog> {
        let stressors_json = serde_json::to_string(&input.stressors).unwrap_or_else(|_| "[]".to_string());
        let time_of_day = input.time_of_day.filter(|t| !t.trim().is_empty()).unwrap_or_else(|| {
            let now = chrono::Local::now();
            let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(12);
            if (5..12).contains(&hour) {
                "morning".to_string()
            } else if (12..17).contains(&hour) {
                "afternoon".to_string()
            } else if (17..21).contains(&hour) {
                "evening".to_string()
            } else {
                "night".to_string()
            }
        });

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE emotional_logs SET 
                    date = ?1, time_of_day = ?2, mood = ?3, energy_level = ?4, 
                    motivation_level = ?5, stress_level = ?6, perceived_recovery = ?7, 
                    stressors_json = ?8, notes = ?9 
                WHERE id = ?10",
                params![
                    input.date,
                    time_of_day,
                    input.mood,
                    input.energy_level,
                    input.motivation_level,
                    input.stress_level,
                    input.perceived_recovery,
                    stressors_json,
                    input.notes,
                    id
                ],
            )?;
            let logs = Self::get_emotional_logs(conn, Some(input.date), None, None)?;
            Ok(logs.into_iter().find(|l| l.id == id).unwrap())
        } else {
            conn.execute(
                "INSERT INTO emotional_logs (
                    date, time_of_day, mood, energy_level, motivation_level, 
                    stress_level, perceived_recovery, stressors_json, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    input.date,
                    time_of_day,
                    input.mood,
                    input.energy_level,
                    input.motivation_level,
                    input.stress_level,
                    input.perceived_recovery,
                    stressors_json,
                    input.notes,
                ],
            )?;
            let last_id = conn.last_insert_rowid();
            let logs = Self::get_emotional_logs(conn, Some(input.date), None, None)?;
            Ok(logs.into_iter().find(|l| l.id == last_id).unwrap())
        }
    }

    pub fn delete_emotional_log(conn: &Connection, id: i64) -> SqlResult<bool> {
        let count = conn.execute("DELETE FROM emotional_logs WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    // --- Calendar Month Aggregation ---
    pub fn get_calendar_month(
        conn: &Connection,
        year: i32,
        month: u32,
    ) -> SqlResult<CalendarMonthResponse> {
        let start_date = format!("{:04}-{:02}-01", year, month);
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        };
        let end_date = format!("{:04}-{:02}-{:02}", year, month, days_in_month);

        let month_names = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];
        let month_name = month_names.get((month - 1) as usize).unwrap_or(&"Month").to_string();

        let today_str: String = conn
            .query_row("SELECT date('now', 'localtime')", [], |r| r.get(0))
            .unwrap_or_else(|_| "2026-08-29".to_string());

        // 1. Fetch biometrics for month
        let mut bio_stmt = conn.prepare(
            "SELECT date, sleep_score, steps, resting_hr, hrv_status, hrv_rmssd, stress_level, active_calories 
             FROM garmin_biometrics 
             WHERE date >= ?1 AND date <= ?2",
        )?;
        let bio_rows = bio_stmt.query_map(params![start_date, end_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, i32>(7)?,
            ))
        })?;
        let mut bio_map = std::collections::HashMap::new();
        for b in bio_rows.flatten() {
            bio_map.insert(b.0.clone(), b);
        }

        // 2. Fetch all activities for month
        let mut act_stmt = conn.prepare(
            "SELECT id, activity_type, title, duration_sec, distance_meters, calories, start_time 
             FROM garmin_activities 
             WHERE date(start_time) >= ?1 AND date(start_time) <= ?2 
             ORDER BY start_time ASC",
        )?;
        let act_rows = act_stmt.query_map(params![start_date, end_date], |row| {
            let start_time: String = row.get(6)?;
            let date = start_time.chars().take(10).collect::<String>();
            let dist_m: f64 = row.get(4)?;
            let dur_s: i32 = row.get(3)?;
            Ok((
                date,
                CalendarActivityPill {
                    id: row.get(0)?,
                    sport: row.get(1)?,
                    title: row.get(2)?,
                    duration_min: (dur_s / 60).max(1),
                    distance_km: (dist_m / 1000.0 * 100.0).round() / 100.0,
                    calories: row.get(5)?,
                    start_time,
                },
            ))
        })?;
        let mut act_map: std::collections::HashMap<String, Vec<CalendarActivityPill>> = std::collections::HashMap::new();
        for a in act_rows.flatten() {
            act_map.entry(a.0).or_default().push(a.1);
        }

        // 3. Fetch all food logs for month
        let all_food_logs = Self::get_food_logs(conn, None, Some(500))?;
        let mut food_map: std::collections::HashMap<String, Vec<FoodLog>> = std::collections::HashMap::new();
        for f in all_food_logs {
            if f.date >= start_date && f.date <= end_date {
                food_map.entry(f.date.clone()).or_default().push(f);
            }
        }

        // 4. Fetch all emotional logs for month
        let all_emotions = Self::get_emotional_logs(conn, None, Some(start_date.clone()), Some(end_date.clone()))?;
        let mut emotion_map: std::collections::HashMap<String, Vec<EmotionalLog>> = std::collections::HashMap::new();
        for e in all_emotions {
            emotion_map.entry(e.date.clone()).or_default().push(e);
        }

        // 5. Fetch all calendar events for month
        let all_events = crate::services::calendar::CalendarService::get_events_for_range(conn, &start_date, &end_date).unwrap_or_default();
        let mut event_map: std::collections::HashMap<String, Vec<crate::services::calendar::CalendarEventItem>> = std::collections::HashMap::new();
        for ev in all_events {
            event_map.entry(ev.date.clone()).or_default().push(ev);
        }

        // 6. Construct each day
        let mut days = Vec::with_capacity(days_in_month as usize);
        for day in 1..=days_in_month {
            let date_str = format!("{:04}-{:02}-{:02}", year, month, day);
            let is_today = date_str == today_str;

            let (sleep_score, steps, resting_hr, hrv_status, hrv_rmssd, stress_level, active_cal) =
                if let Some(b) = bio_map.get(&date_str) {
                    (b.1, b.2, b.3, b.4.clone(), b.5, b.6, b.7)
                } else {
                    (0, 0, 0, "No Data".to_string(), 0, 0, 0)
                };

            let activities = act_map.remove(&date_str).unwrap_or_default();
            let foods = food_map.remove(&date_str).unwrap_or_default();
            let emotions = emotion_map.remove(&date_str).unwrap_or_default();
            let events = event_map.remove(&date_str).unwrap_or_default();

            let total_food_calories: i32 = foods.iter().map(|f| f.calories).sum();
            let total_active_calories: i32 = active_cal + activities.iter().map(|a| a.calories).sum::<i32>();
            let latest_mood = emotions.first().map(|e| e.mood.clone());
            
            let avg_emotional_stress = if !emotions.is_empty() {
                let sum: i32 = emotions.iter().map(|e| e.stress_level).sum();
                Some(((sum as f64) / (emotions.len() as f64) * 10.0).round() / 10.0)
            } else {
                None
            };

            let avg_energy_level = if !emotions.is_empty() {
                let sum: i32 = emotions.iter().map(|e| e.energy_level).sum();
                Some(((sum as f64) / (emotions.len() as f64) * 10.0).round() / 10.0)
            } else {
                None
            };

            days.push(CalendarDaySummary {
                date: date_str,
                day_of_month: day,
                is_current_month: true,
                is_today,
                sleep_score,
                steps,
                resting_hr,
                hrv_status,
                hrv_rmssd,
                stress_level,
                total_food_calories,
                total_active_calories,
                food_count: foods.len(),
                activity_count: activities.len(),
                latest_mood,
                avg_emotional_stress,
                avg_energy_level,
                activities,
                foods,
                emotions,
                events,
            });
        }

        Ok(CalendarMonthResponse {
            year,
            month,
            month_name,
            days,
        })
    }
}
