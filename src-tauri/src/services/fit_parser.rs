use crate::services::garmin::GarminActivity;
use crate::services::goblin;
use fitparser::Value;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFitActivity {
    pub file_type: String,
    pub manufacturer: String,
    pub product: String,
    pub serial_number: Option<u64>,
    pub sport: String,
    pub subsport: Option<String>,
    pub title: String,
    pub start_time: String,
    pub duration_sec: u32,
    pub distance_meters: f64,
    pub calories: u32,
    pub avg_hr: u32,
    pub max_hr: u32,
    pub avg_cadence: Option<u32>,
    pub avg_speed_mps: Option<f32>,
    pub max_speed_mps: Option<f32>,
    pub total_ascent_m: Option<f32>,
    pub total_descent_m: Option<f32>,
    pub aerobic_training_effect: Option<f32>,
    pub anaerobic_training_effect: Option<f32>,
    pub gps_track: Vec<[f64; 2]>,
    pub total_steps: Option<u32>,
    pub total_records: usize,
    pub has_session: bool,
    pub body_battery: Option<u32>,
    pub stress_level: Option<u32>,
    pub resting_hr: Option<u32>,
    pub sleep_score: Option<u32>,
    pub sleep_duration_sec: Option<u32>,
    pub sleep_deep_sec: Option<u32>,
    pub sleep_light_sec: Option<u32>,
    pub sleep_rem_sec: Option<u32>,
    pub sleep_awake_sec: Option<u32>,
    pub hrv_rmssd: Option<u32>,
    pub hrv_status: Option<String>,
    pub respiration_rate: Option<u32>,
    pub battery_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitImportResult {
    pub success: bool,
    pub activity: GarminActivity,
    pub xp_earned: i64,
    pub gold_earned: i64,
    pub gps_points_count: usize,
    pub status_message: String,
}

pub fn fit_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn value_to_f64(val: &Value) -> Option<f64> {
    match val {
        Value::Float32(f) => Some(*f as f64),
        Value::Float64(f) => Some(*f),
        Value::UInt8(u) => Some(*u as f64),
        Value::UInt16(u) => Some(*u as f64),
        Value::UInt32(u) => Some(*u as f64),
        Value::UInt64(u) => Some(*u as f64),
        Value::SInt8(i) => Some(*i as f64),
        Value::SInt16(i) => Some(*i as f64),
        Value::SInt32(i) => Some(*i as f64),
        Value::SInt64(i) => Some(*i as f64),
        Value::Enum(e) => Some(*e as f64),
        Value::Array(arr) => arr.first().and_then(value_to_f64),
        _ => None,
    }
}

fn value_to_f64_vec(val: &Value) -> Vec<f64> {
    match val {
        Value::Array(arr) => arr.iter().filter_map(value_to_f64).collect(),
        _ => value_to_f64(val).into_iter().collect(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct Timestamp16Decoder {
    pub last_timestamp: u32,
}

impl Timestamp16Decoder {
    pub fn new() -> Self {
        Self { last_timestamp: 0 }
    }

    pub fn update_32(&mut self, ts: u32) {
        self.last_timestamp = ts;
    }

    pub fn decode_16(&mut self, ts16: u16) -> u32 {
        let mut calculated = (self.last_timestamp & 0xFFFF_0000) | (ts16 as u32);
        if calculated < self.last_timestamp {
            calculated += 0x10000;
        }
        self.last_timestamp = calculated;
        calculated
    }
}

fn value_to_u32(val: &Value) -> Option<u32> {
    match val {
        Value::UInt8(u) => Some(*u as u32),
        Value::UInt16(u) => Some(*u as u32),
        Value::UInt32(u) => Some(*u),
        Value::UInt64(u) => Some(*u as u32),
        Value::SInt8(i) => Some((*i).max(0) as u32),
        Value::SInt16(i) => Some((*i).max(0) as u32),
        Value::SInt32(i) => Some((*i).max(0) as u32),
        Value::Float32(f) => Some((*f).max(0.0) as u32),
        Value::Float64(f) => Some((*f).max(0.0) as u32),
        Value::Enum(e) => Some(*e as u32),
        Value::Array(arr) => arr.first().and_then(value_to_u32),
        _ => None,
    }
}

fn value_to_epoch_sec(val: &Value) -> Option<i64> {
    match val {
        Value::Timestamp(dt) => Some(dt.timestamp()),
        Value::UInt32(u) => {
            if *u > 600_000_000 && *u < 2_000_000_000 {
                Some(*u as i64)
            } else if *u > 0 && *u < 600_000_000 {
                Some(*u as i64 + 631_065_600)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn sport_value_to_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.clone()),
        Value::Enum(e) => {
            let s = match *e {
                1 => "Running",
                2 => "Cycling",
                3 => "Transition",
                4 => "Fitness Equipment",
                5 => "Swimming",
                6 => "Basketball",
                7 => "Soccer",
                8 => "Tennis",
                10 => "Training",
                11 => "Walking",
                12 => "Cross Country Skiing",
                13 => "Alpine Skiing",
                14 => "Snowboarding",
                15 => "Rowing",
                16 => "Mountaineering",
                17 => "Hiking",
                18 => "Multisport",
                19 => "Paddling",
                20 => "Flying",
                21 => "E-Biking",
                25 => "Golf",
                31 => "Rock Climbing",
                32 => "Sailing",
                37 => "SUP",
                38 => "Surfing",
                47 => "Boxing",
                48 => "Floor Climbing",
                53 => "HIIT",
                62 => "Pilates",
                67 => "Yoga",
                83 => "Breathwork",
                _ => "Workout",
            };
            Some(s.to_string())
        }
        Value::Array(arr) => arr.first().and_then(sport_value_to_string),
        _ => None,
    }
}

fn value_to_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.clone()),
        Value::Timestamp(dt) => {
            Some(dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string())
        }
        Value::Enum(e) => Some(format!("{}", e)),
        Value::Array(arr) => arr.first().and_then(value_to_string),
        _ => None,
    }
}

pub fn semicircles_to_degrees(semicircles: f64) -> f64 {
    semicircles * (180.0 / 2147483648.0)
}

pub fn parse_fit_bytes(bytes: &[u8]) -> Result<ParsedFitActivity, String> {
    if bytes.len() < 14 {
        return Err(format!("File too small to be a valid FIT file (got {} bytes)", bytes.len()));
    }

    let records = match fitparser::from_bytes(bytes) {
        Ok(r) => r,
        Err(direct_err) => {
            let mut found_records = None;
            for i in 0..bytes.len().saturating_sub(14) {
                if &bytes[i + 8..i + 12] == b".FIT" {
                    if let Ok(r) = fitparser::from_bytes(&bytes[i..]) {
                        found_records = Some(r);
                        break;
                    }
                }
            }
            match found_records {
                Some(r) => r,
                None => return Err(format!("FIT parsing failed: {}", direct_err)),
            }
        }
    };

    let mut has_session = false;
    let mut sport_name = "Workout".to_string();
    let mut subsport_name: Option<String> = None;
    let mut start_time_str = String::new();
    let mut duration_sec: u32 = 0;
    let mut distance_meters: f64 = 0.0;
    let mut calories: u32 = 0;
    let mut avg_hr: u32 = 0;
    let mut max_hr: u32 = 0;
    let mut avg_cadence: Option<u32> = None;
    let mut avg_speed: Option<f32> = None;
    let mut max_speed: Option<f32> = None;
    let mut total_ascent: Option<f32> = None;
    let mut total_descent: Option<f32> = None;
    let mut aerobic_te: Option<f32> = None;
    let mut anaerobic_te: Option<f32> = None;
    let mut gps_points: Vec<[f64; 2]> = Vec::new();
    let mut total_records = 0;
    let mut serial_number: Option<u64> = None;
    let mut product_name = "Garmin Device".to_string();
    let mut manufacturer_name = "Garmin".to_string();

    let mut monitoring_steps: u32 = 0;
    let mut monitoring_calories: u32 = 0;
    let mut monitoring_hr: Option<u32> = None;
    let mut resting_hr: Option<u32> = None;
    let mut sleep_score: Option<u32> = None;
    let mut sleep_duration_sec: Option<u32> = None;
    let mut sleep_deep_sec: Option<u32> = None;
    let mut sleep_light_sec: Option<u32> = None;
    let mut sleep_rem_sec: Option<u32> = None;
    let mut sleep_awake_sec: Option<u32> = None;
    let mut hrv_rmssd: Option<u32> = None;
    let mut hrv_status: Option<String> = None;
    let mut stress_level: Option<u32> = None;
    let mut body_battery: Option<u32> = None;
    let mut respiration_rate: Option<u32> = None;
    let mut battery_level: Option<u32> = None;

    let mut hr_sum: u64 = 0;
    let mut hr_count: u64 = 0;
    let mut hrv_sum: f64 = 0.0;
    let mut hrv_count: u32 = 0;
    let mut raw_rr_intervals: Vec<f64> = Vec::new();
    let mut ts_decoder = Timestamp16Decoder::new();
    let mut max_speed_val: f32 = 0.0;
    let mut speed_sum: f64 = 0.0;
    let mut speed_count: u64 = 0;
    let mut max_record_dist: f64 = 0.0;
    let mut max_record_cal: u32 = 0;
    let mut sleep_stages: Vec<(String, i64)> = Vec::new();

    for (idx, record) in records.iter().enumerate() {
        let mesg_name = format!("{:?}", record.kind()).to_lowercase();
        if idx < 5 || mesg_name.contains("session") || mesg_name.contains("activity") || mesg_name.contains("lap") || mesg_name.contains("hrv") || mesg_name.contains("sleep") || mesg_name.contains("monitor") {
            eprintln!("[GarminGoblin FIT] Rec #{}: kind='{}' fields={}", idx, mesg_name, record.fields().len());
            for f in record.fields() {
                eprintln!("[GarminGoblin FIT]   Field '{}' = {:?}", f.name(), f.value());
            }
        }

        if mesg_name.contains("session") {
            has_session = true;
            for field in record.fields() {
                match field.name() {
                    "sport" => {
                        if let Some(s) = sport_value_to_string(field.value()) {
                            sport_name = s;
                        }
                    }
                    "sub_sport" => {
                        subsport_name = sport_value_to_string(field.value());
                    }
                    "start_time" | "timestamp" => {
                        if let Some(ts) = value_to_string(field.value()) {
                            if start_time_str.is_empty() {
                                start_time_str = ts.replace('T', " ").replace('Z', "").chars().take(19).collect();
                            }
                        }
                    }
                    "total_elapsed_time" | "total_timer_time" => {
                        if let Some(dur) = value_to_f64(field.value()) {
                            if dur > 0.0 {
                                duration_sec = dur as u32;
                            }
                        }
                    }
                    "total_distance" => {
                        if let Some(dist) = value_to_f64(field.value()) {
                            if dist > 0.0 {
                                distance_meters = dist;
                            }
                        }
                    }
                    "total_calories" => {
                        if let Some(cal) = value_to_u32(field.value()) {
                            calories = cal;
                        }
                    }
                    "avg_heart_rate" => {
                        if let Some(hr) = value_to_u32(field.value()) {
                            avg_hr = hr;
                        }
                    }
                    "max_heart_rate" => {
                        if let Some(hr) = value_to_u32(field.value()) {
                            max_hr = hr;
                        }
                    }
                    "avg_cadence" => {
                        avg_cadence = value_to_u32(field.value());
                    }
                    "avg_speed" | "enhanced_avg_speed" => {
                        if let Some(s) = value_to_f64(field.value()) {
                            avg_speed = Some(s as f32);
                        }
                    }
                    "max_speed" | "enhanced_max_speed" => {
                        if let Some(s) = value_to_f64(field.value()) {
                            max_speed = Some(s as f32);
                        }
                    }
                    "total_ascent" => {
                        if let Some(a) = value_to_f64(field.value()) {
                            total_ascent = Some(a as f32);
                        }
                    }
                    "total_descent" => {
                        if let Some(d) = value_to_f64(field.value()) {
                            total_descent = Some(d as f32);
                        }
                    }
                    "total_training_effect" => {
                        if let Some(te) = value_to_f64(field.value()) {
                            aerobic_te = Some(te as f32);
                        }
                    }
                    "total_anaerobic_effect" | "total_anaerobic_training_effect" => {
                        if let Some(te) = value_to_f64(field.value()) {
                            anaerobic_te = Some(te as f32);
                        }
                    }
                    _ => {}
                }
            }
        }
        else if mesg_name.contains("lap") {
            for field in record.fields() {
                match field.name() {
                    "total_elapsed_time" | "total_timer_time" if duration_sec == 0 => {
                        if let Some(dur) = value_to_f64(field.value()) {
                            duration_sec = dur as u32;
                        }
                    }
                    "total_distance" if distance_meters == 0.0 => {
                        if let Some(dist) = value_to_f64(field.value()) {
                            distance_meters = dist;
                        }
                    }
                    "total_calories" if calories == 0 => {
                        if let Some(cal) = value_to_u32(field.value()) {
                            calories = cal;
                        }
                    }
                    "avg_heart_rate" if avg_hr == 0 => {
                        if let Some(hr) = value_to_u32(field.value()) {
                            avg_hr = hr;
                        }
                    }
                    _ => {}
                }
            }
        }
        else if mesg_name.contains("record") {
            total_records += 1;
            let mut lat: Option<f64> = None;
            let mut lon: Option<f64> = None;

            for field in record.fields() {
                match field.name() {
                    "position_lat" => {
                        if let Some(v) = value_to_f64(field.value()) {
                            let deg = if v.abs() > 90.0 { semicircles_to_degrees(v) } else { v };
                            if deg.abs() <= 90.0 { lat = Some(deg); }
                        }
                    }
                    "position_long" => {
                        if let Some(v) = value_to_f64(field.value()) {
                            let deg = if v.abs() > 180.0 { semicircles_to_degrees(v) } else { v };
                            if deg.abs() <= 180.0 { lon = Some(deg); }
                        }
                    }
                    "heart_rate" => {
                        if let Some(hr) = value_to_u32(field.value()) {
                            if hr > 30 && hr < 240 {
                                hr_sum += hr as u64;
                                hr_count += 1;
                                max_hr = max_hr.max(hr);
                                if avg_hr == 0 { avg_hr = hr; }
                            }
                        }
                    }
                    "distance" => {
                        if let Some(d) = value_to_f64(field.value()) {
                            max_record_dist = max_record_dist.max(d);
                        }
                    }
                    "speed" | "enhanced_speed" => {
                        if let Some(s) = value_to_f64(field.value()) {
                            if s > 0.0 && s < 100.0 {
                                speed_sum += s;
                                speed_count += 1;
                                max_speed_val = max_speed_val.max(s as f32);
                            }
                        }
                    }
                    "calories" => {
                        if let Some(c) = value_to_u32(field.value()) {
                            max_record_cal = max_record_cal.max(c);
                        }
                    }
                    "timestamp" => {
                        if let Some(u) = value_to_u32(field.value()) {
                            ts_decoder.update_32(u);
                        }
                    }
                    "timestamp_16" => {
                        if let Some(u16_val) = value_to_u32(field.value()) {
                            let _ = ts_decoder.decode_16(u16_val as u16);
                        }
                    }
                    _ => {}
                }
            }

            if let (Some(la), Some(lo)) = (lat, lon) {
                if la != 0.0 || lo != 0.0 {
                    gps_points.push([la, lo]);
                }
            }
        }
        else if !mesg_name.contains("workout") && !mesg_name.contains("schedule") {
            for field in record.fields() {
                let fname = field.name().to_lowercase();
                if (fname == "steps" || fname == "total_steps" || fname == "cumulative_steps" || fname == "cycles")
                    && !fname.contains("valid") && !fname.contains("step_name") {
                    if let Some(s) = value_to_u32(field.value()) {
                        if s > 0 && s < 200_000 {
                            monitoring_steps = monitoring_steps.max(s);
                        }
                    }
                }
                if fname.contains("calorie") && !fname.contains("target") {
                    if let Some(c) = value_to_u32(field.value()) { monitoring_calories = monitoring_calories.max(c); }
                }
                if fname == "heart_rate" || fname == "current_heart_rate" {
                    if let Some(hr) = value_to_u32(field.value()) {
                        if hr > 30 && hr < 240 { monitoring_hr = Some(hr); }
                    }
                }
                if fname.contains("current_day_resting_heart_rate") || fname.contains("current_day_resting_hr") {
                    if let Some(rhr) = value_to_u32(field.value()) {
                        if (30..150).contains(&rhr) { resting_hr = Some(rhr); }
                    }
                } else if fname == "resting_heart_rate" || fname == "resting_hr" {
                    if let Some(rhr) = value_to_u32(field.value()) {
                        if (30..150).contains(&rhr) && resting_hr.is_none() { resting_hr = Some(rhr); }
                    }
                }
                if fname.contains("body_battery") || fname.contains("battery_charge") || fname == "charge" {
                    if let Some(bb) = value_to_u32(field.value()) { 
                        if (1..=100).contains(&bb) { body_battery = Some(bb); }
                    }
                }
                if fname.contains("stress") && !fname.contains("qualifier") {
                    if let Some(sl) = value_to_u32(field.value()) { 
                        if (1..=100).contains(&sl) { stress_level = Some(sl); }
                    }
                }
                if fname.contains("respiration") {
                    if let Some(rr) = value_to_u32(field.value()) { respiration_rate = Some(rr); }
                }
                if fname == "battery_level" || fname == "battery_percent" || fname == "battery_percentage" {
                    if let Some(b) = value_to_u32(field.value()) {
                        if (1..=100).contains(&b) { battery_level = Some(b); }
                    }
                }
                if fname == "overall_sleep_score" || fname == "sleep_score" {
                    if let Some(ss) = value_to_u32(field.value()) {
                        if (1..=100).contains(&ss) { sleep_score = Some(ss); }
                    }
                } else if fname.contains("sleep_score") && sleep_score.is_none() {
                    let parsed_ss = value_to_u32(field.value()).filter(|v| (1..=100).contains(v));
                    if parsed_ss.is_some() { sleep_score = parsed_ss; }
                }
                if mesg_name.contains("sleep_level") || mesg_name.contains("sleeplevel") {
                    let mut st_name = String::new();
                    let mut st_ts = 0i64;
                    for f in record.fields() {
                        let fn_lower = f.name().to_lowercase();
                        if fn_lower == "sleep_level" || fn_lower == "level" {
                            if let Some(s) = value_to_string(f.value()) { st_name = s.to_lowercase(); }
                        }
                        if fn_lower == "timestamp" {
                            if let Some(ts) = value_to_epoch_sec(f.value()) { st_ts = ts; }
                        }
                    }
                    if !st_name.is_empty() && st_ts > 0 {
                        sleep_stages.push((st_name, st_ts));
                    }
                }
                if mesg_name.contains("napevent") || mesg_name.contains("nap") {
                    let mut s_ts = 0i64;
                    let mut e_ts = 0i64;
                    for f in record.fields() {
                        let fn_lower = f.name().to_lowercase();
                        if fn_lower == "start_time" {
                            if let Some(ts) = value_to_epoch_sec(f.value()) { s_ts = ts; }
                        }
                        if fn_lower == "end_time" {
                            if let Some(ts) = value_to_epoch_sec(f.value()) { e_ts = ts; }
                        }
                    }
                    if e_ts > s_ts {
                        let nap_sec = (e_ts - s_ts) as u32;
                        sleep_duration_sec = Some(sleep_duration_sec.unwrap_or(0) + nap_sec);
                    }
                }
                if fname == "total_sleep_time" || fname == "sleep_duration" {
                    if let Some(sec) = value_to_u32(field.value()) { sleep_duration_sec = Some(sec); }
                }
                if fname.contains("deep") && fname.contains("duration") {
                    if let Some(sec) = value_to_u32(field.value()) { sleep_deep_sec = Some(sec); }
                }
                if fname.contains("light") && fname.contains("duration") {
                    if let Some(sec) = value_to_u32(field.value()) { sleep_light_sec = Some(sec); }
                }
                if fname.contains("rem") && fname.contains("duration") {
                    if let Some(sec) = value_to_u32(field.value()) { sleep_rem_sec = Some(sec); }
                }
                if fname.contains("awake") && fname.contains("duration") {
                    if let Some(sec) = value_to_u32(field.value()) { sleep_awake_sec = Some(sec); }
                }
                if fname == "rmssd" || fname == "weekly_average" || fname == "last_night_average" {
                    if let Some(v) = value_to_u32(field.value()) { if v > 0 { hrv_rmssd = Some(v); } }
                }
                if (fname == "value" || fname == "time" || fname == "rr_interval") && mesg_name.contains("hrv") {
                    let rrs = value_to_f64_vec(field.value());
                    for r in rrs {
                        let ms = if r < 5.0 { r * 1000.0 } else { r };
                        if (200.0..2500.0).contains(&ms) {
                            raw_rr_intervals.push(ms);
                        } else if (10.0..250.0).contains(&r) {
                            hrv_sum += r;
                            hrv_count += 1;
                        }
                    }
                }
                if fname == "status" && mesg_name.contains("hrv") {
                    if let Some(st) = value_to_string(field.value()) {
                        if !st.is_empty() && st != "No Data" && st != "none" {
                            hrv_status = Some(st);
                        }
                    }
                }
                if fname == "product_name" || fname == "product" {
                    if let Some(p) = value_to_string(field.value()) { product_name = p; }
                }
                if fname == "manufacturer" {
                    if let Some(m) = value_to_string(field.value()) { manufacturer_name = m; }
                }
                if fname == "serial_number" {
                    if let Some(sn) = value_to_u32(field.value()) { serial_number = Some(sn as u64); }
                }
                if fname == "timestamp" && start_time_str.is_empty() {
                    if let Some(ts) = value_to_string(field.value()) {
                        start_time_str = ts.replace('T', " ").replace('Z', "").chars().take(19).collect();
                    }
                }
            }
        }
    }

    if sleep_stages.len() >= 2 {
        sleep_stages.sort_by_key(|s| s.1);
        let mut deep_acc = 0u32;
        let mut rem_acc = 0u32;
        let mut light_acc = 0u32;
        let mut awake_acc = 0u32;

        for w in sleep_stages.windows(2) {
            let stage = &w[0].0;
            let t0 = w[0].1;
            let t1 = w[1].1;
            if t1 > t0 {
                let diff = (t1 - t0).clamp(0, 21600) as u32; // Allow normal physiological stage blocks
                if stage.contains("deep") {
                    deep_acc += diff;
                } else if stage.contains("rem") {
                    rem_acc += diff;
                } else if stage.contains("light") {
                    light_acc += diff;
                } else if stage.contains("awake") {
                    awake_acc += diff;
                }
            }
        }

        if deep_acc > 0 { sleep_deep_sec = Some(deep_acc); }
        if rem_acc > 0 { sleep_rem_sec = Some(rem_acc); }
        if light_acc > 0 { sleep_light_sec = Some(light_acc); }
        if awake_acc > 0 { sleep_awake_sec = Some(awake_acc); }

        let t_first = sleep_stages.first().map(|s| s.1).unwrap_or(0);
        let t_last = sleep_stages.last().map(|s| s.1).unwrap_or(0);
        let total_window = if t_last > t_first { (t_last - t_first) as u32 } else { 0 };

        let total_calc = deep_acc + rem_acc + light_acc;
        if total_calc > 0 {
            sleep_duration_sec = Some(total_calc.min(54000));
        } else if total_window > awake_acc {
            sleep_duration_sec = Some((total_window - awake_acc).min(54000));
        }
    }

    if distance_meters == 0.0 && max_record_dist > 0.0 { distance_meters = max_record_dist; }
    if calories == 0 && max_record_cal > 0 { calories = max_record_cal; }
    if avg_hr == 0 && hr_count > 0 { avg_hr = (hr_sum / hr_count) as u32; }
    if avg_speed.is_none() && speed_count > 0 { avg_speed = Some((speed_sum / speed_count as f64) as f32); }
    if max_speed.is_none() && max_speed_val > 0.0 { max_speed = Some(max_speed_val); }
    if avg_hr == 0 { if let Some(hr) = monitoring_hr { avg_hr = hr; } }
    if hrv_rmssd.is_none() && raw_rr_intervals.len() >= 2 {
        let mut diff_sq_sum = 0.0;
        let count = raw_rr_intervals.len() - 1;
        for w in raw_rr_intervals.windows(2) {
            let diff = w[1] - w[0];
            diff_sq_sum += diff * diff;
        }
        let calc_rmssd = (diff_sq_sum / count as f64).sqrt().round() as u32;
        if (10..250).contains(&calc_rmssd) {
            hrv_rmssd = Some(calc_rmssd);
        }
    }
    if hrv_rmssd.is_none() && hrv_count > 0 {
        let avg_rmssd = (hrv_sum / hrv_count as f64).round() as u32;
        hrv_rmssd = Some(avg_rmssd);
    }
    if hrv_status.is_none() || hrv_status.as_deref() == Some("none") || hrv_status.as_deref() == Some("No Data") || hrv_status.as_deref() == Some("") {
        if let Some(rmssd) = hrv_rmssd {
            if rmssd > 0 {
                hrv_status = Some(if rmssd >= 45 {
                    "Balanced".to_string()
                } else if rmssd >= 30 {
                    "Unbalanced".to_string()
                } else {
                    "Low".to_string()
                });
            }
        }
    }
    if body_battery.is_none() {
        if let (Some(slp), Some(stress)) = (sleep_score, stress_level) {
            let hrv_bonus = hrv_rmssd.unwrap_or(45) as f64 * 0.25;
            let calc_bb = ((slp as f64 * 0.8) - (stress as f64 * 0.35) + hrv_bonus).clamp(15.0, 98.0).round() as u32;
            body_battery = Some(calc_bb);
        } else if let Some(slp) = sleep_score {
            body_battery = Some(slp.clamp(20, 95));
        }
    }

    if start_time_str.is_empty() { start_time_str = chrono_or_system_time(); }

    let title = if has_session {
        format!("Garmin {}", sport_name)
    } else if monitoring_steps > 0 {
        "Garmin Daily Activity".to_string()
    } else if sleep_duration_sec.is_some() {
        "Garmin Sleep Tracking".to_string()
    } else if hrv_rmssd.is_some() {
        "Garmin Overnight HRV".to_string()
    } else {
        format!("Garmin {}", sport_name)
    };

    Ok(ParsedFitActivity {
        file_type: "FIT Activity".to_string(),
        manufacturer: manufacturer_name,
        product: product_name,
        serial_number,
        sport: sport_name,
        subsport: subsport_name,
        title,
        start_time: start_time_str,
        duration_sec,
        distance_meters,
        calories,
        avg_hr,
        max_hr,
        avg_cadence,
        avg_speed_mps: avg_speed,
        max_speed_mps: max_speed,
        total_ascent_m: total_ascent,
        total_descent_m: total_descent,
        aerobic_training_effect: aerobic_te,
        anaerobic_training_effect: anaerobic_te,
        gps_track: gps_points,
        total_steps: if monitoring_steps > 0 { Some(monitoring_steps) } else { None },
        total_records,
        has_session,
        body_battery,
        stress_level,
        hrv_status,
        hrv_rmssd,
        sleep_score,
        sleep_duration_sec,
        sleep_deep_sec,
        sleep_light_sec,
        sleep_rem_sec,
        sleep_awake_sec,
        respiration_rate,
        resting_hr,
        battery_level,
    })
}

pub fn ingest_parsed_fit(
    db: Arc<Mutex<Connection>>,
    parsed: ParsedFitActivity,
    raw_fit_bytes: Option<&[u8]>,
) -> Result<FitImportResult, String> {
    let raw_fit_hash = raw_fit_bytes.map(fit_sha256);
    let mut conn = db.lock().map_err(|_| "DB lock failed")?;

    let is_workout = parsed.has_session
        || (parsed.distance_meters > 50.0 && parsed.duration_sec > 60)
        || (parsed.duration_sec > 180 && parsed.avg_hr > 0);

    if let Some(ref hash) = raw_fit_hash {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM garmin_activities WHERE fit_sha256 = ?1 LIMIT 1",
                [hash],
                |r| r.get(0),
            )
            .ok();

        if existing.is_some() {
            eprintln!("[GarminGoblin DB] Duplicate FIT hash detected (id={:?}), skipping insertion: {}", existing, hash);
            let activity = GarminActivity {
                id: existing.unwrap_or(0),
                activity_type: parsed.sport.clone(),
                title: parsed.title.clone(),
                start_time: parsed.start_time.clone(),
                duration_sec: parsed.duration_sec as i32,
                distance_meters: parsed.distance_meters,
                calories: parsed.calories as i32,
                avg_hr: parsed.avg_hr as i32,
                max_hr: parsed.max_hr as i32,
                aerobic_training_effect: parsed.aerobic_training_effect.unwrap_or(0.0) as f64,
                anaerobic_training_effect: parsed.anaerobic_training_effect.unwrap_or(0.0) as f64,
            };
            return Ok(FitImportResult {
                success: true,
                activity,
                xp_earned: 0,
                gold_earned: 0,
                gps_points_count: parsed.gps_track.len(),
                status_message: "This FIT file was already imported; no duplicate rewards or biometrics added.".to_string(),
            });
        }
    }

    let fit_date = if parsed.start_time.len() >= 10 {
        parsed.start_time[0..10].to_string()
    } else {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    };

    let current_hr = if parsed.avg_hr > 0 { parsed.avg_hr } else { 0 };
    let resting_hr = parsed.resting_hr.unwrap_or(0);
    let body_battery = parsed.body_battery.unwrap_or(0);
    let stress = parsed.stress_level.unwrap_or(0);
    let hrv_status = parsed.hrv_status.clone().unwrap_or_default();
    let hrv_rmssd = parsed.hrv_rmssd.unwrap_or(0);

    let sleep_score = parsed.sleep_score.unwrap_or(0);
    let sleep_dur = parsed.sleep_duration_sec.unwrap_or(0);
    let sleep_deep = parsed.sleep_deep_sec.unwrap_or(0);
    let sleep_rem = parsed.sleep_rem_sec.unwrap_or(0);
    let sleep_light = parsed.sleep_light_sec.unwrap_or(0);
    let sleep_awake = parsed.sleep_awake_sec.unwrap_or(0);
    let respiration_rate = parsed.respiration_rate.unwrap_or(0);

    let steps_actual = parsed.total_steps.unwrap_or(0);

    let has_any_biometrics = steps_actual > 0 
        || parsed.calories > 0 
        || current_hr > 0 
        || resting_hr > 0 
        || body_battery > 0 
        || stress > 0 
        || hrv_rmssd > 0 
        || sleep_score > 0 
        || sleep_dur > 0;

    if !is_workout && !has_any_biometrics {
        eprintln!("[GarminGoblin DB] Skipping ingestion for non-workout template FIT without biometrics: title='{}'", parsed.title);
        let activity = GarminActivity {
            id: 0,
            activity_type: parsed.sport.clone(),
            title: parsed.title.clone(),
            start_time: parsed.start_time.clone(),
            duration_sec: parsed.duration_sec as i32,
            distance_meters: parsed.distance_meters,
            calories: parsed.calories as i32,
            avg_hr: parsed.avg_hr as i32,
            max_hr: parsed.max_hr as i32,
            aerobic_training_effect: parsed.aerobic_training_effect.unwrap_or(0.0) as f64,
            anaerobic_training_effect: parsed.anaerobic_training_effect.unwrap_or(0.0) as f64,
        };
        return Ok(FitImportResult {
            success: true,
            activity,
            xp_earned: 0,
            gold_earned: 0,
            gps_points_count: 0,
            status_message: "Template FIT parsed without biometric impact.".to_string(),
        });
    }

    eprintln!("[GarminGoblin DB] Ingesting FIT into SQLite: date={}, is_workout={}, steps={}, cal={}, hr={}, rest_hr={}, bb={}, stress={}, hrv='{}' (rmssd={}), sleep_score={}, sleep_dur={}",
        fit_date, is_workout, steps_actual, parsed.calories, current_hr, resting_hr, body_battery, stress, hrv_status, hrv_rmssd, sleep_score, sleep_dur);

    let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {}", e))?;

    let mut inserted_activity_id: i64 = 0;

    if is_workout {
        tx.execute(
            "INSERT INTO garmin_activities (
                activity_type, title, start_time, duration_sec, distance_meters,
                calories, avg_hr, max_hr, aerobic_training_effect, anaerobic_training_effect, fit_sha256
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                &parsed.sport,
                &parsed.title,
                &parsed.start_time,
                parsed.duration_sec as i32,
                parsed.distance_meters,
                parsed.calories as i32,
                parsed.avg_hr as i32,
                parsed.max_hr as i32,
                parsed.aerobic_training_effect.unwrap_or(0.0) as f64,
                parsed.anaerobic_training_effect.unwrap_or(0.0) as f64,
                raw_fit_hash,
            ],
        ).map_err(|e| {
            eprintln!("[GarminGoblin] Failed to insert activity into garmin_activities: {}", e);
            format!("Database insert failed: {}", e)
        })?;
        inserted_activity_id = tx.last_insert_rowid();
    }

    tx.execute(
        "INSERT INTO garmin_biometrics (
            date, steps, active_calories, total_calories, current_hr, resting_hr,
            body_battery, stress_level, hrv_status, hrv_rmssd, sleep_score,
            sleep_duration_sec, sleep_deep_sec, sleep_rem_sec, sleep_light_sec,
            sleep_awake_sec, respiration_rate
        ) VALUES (
            ?1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
        ) ON CONFLICT(date) DO UPDATE SET
            steps = CASE WHEN excluded.steps > 0 THEN MAX(garmin_biometrics.steps, excluded.steps) ELSE garmin_biometrics.steps END,
            active_calories = CASE WHEN ?17 THEN garmin_biometrics.active_calories + excluded.active_calories ELSE MAX(garmin_biometrics.active_calories, excluded.active_calories) END,
            total_calories = CASE WHEN ?17 THEN garmin_biometrics.total_calories + excluded.total_calories ELSE MAX(garmin_biometrics.total_calories, excluded.total_calories) END,
            current_hr = CASE WHEN excluded.current_hr > 0 THEN excluded.current_hr ELSE garmin_biometrics.current_hr END,
            resting_hr = CASE WHEN excluded.resting_hr > 0 THEN excluded.resting_hr ELSE garmin_biometrics.resting_hr END,
            body_battery = CASE WHEN excluded.body_battery > 0 THEN excluded.body_battery ELSE garmin_biometrics.body_battery END,
            stress_level = CASE WHEN excluded.stress_level > 0 THEN excluded.stress_level ELSE garmin_biometrics.stress_level END,
            hrv_status = CASE WHEN excluded.hrv_status != '' AND excluded.hrv_status != 'No Data' AND excluded.hrv_status != 'none' THEN excluded.hrv_status ELSE garmin_biometrics.hrv_status END,
            hrv_rmssd = CASE WHEN excluded.hrv_rmssd > 0 THEN excluded.hrv_rmssd ELSE garmin_biometrics.hrv_rmssd END,
            sleep_score = CASE WHEN excluded.sleep_score > 0 THEN excluded.sleep_score ELSE garmin_biometrics.sleep_score END,
            sleep_duration_sec = CASE WHEN excluded.sleep_duration_sec > 0 THEN excluded.sleep_duration_sec ELSE garmin_biometrics.sleep_duration_sec END,
            sleep_deep_sec = CASE WHEN excluded.sleep_deep_sec > 0 THEN excluded.sleep_deep_sec ELSE garmin_biometrics.sleep_deep_sec END,
            sleep_rem_sec = CASE WHEN excluded.sleep_rem_sec > 0 THEN excluded.sleep_rem_sec ELSE garmin_biometrics.sleep_rem_sec END,
            sleep_light_sec = CASE WHEN excluded.sleep_light_sec > 0 THEN excluded.sleep_light_sec ELSE garmin_biometrics.sleep_light_sec END,
            sleep_awake_sec = CASE WHEN excluded.sleep_awake_sec > 0 THEN excluded.sleep_awake_sec ELSE garmin_biometrics.sleep_awake_sec END,
            respiration_rate = CASE WHEN excluded.respiration_rate > 0 THEN excluded.respiration_rate ELSE garmin_biometrics.respiration_rate END",
        rusqlite::params![
            &fit_date,
            steps_actual,
            parsed.calories,
            current_hr,
            resting_hr,
            body_battery,
            stress,
            &hrv_status,
            hrv_rmssd,
            sleep_score,
            sleep_dur,
            sleep_deep,
            sleep_rem,
            sleep_light,
            sleep_awake,
            respiration_rate,
            is_workout,
        ],
    ).map_err(|e| {
        eprintln!("[GarminGoblin] Failed to upsert garmin_biometrics: {}", e);
        format!("Database upsert failed: {}", e)
    })?;

    if let Some(b) = parsed.battery_level {
        let _ = tx.execute(
            "UPDATE garmin_devices SET battery_level = ?1 WHERE is_paired = 1",
            rusqlite::params![b as i64],
        );
    }

    tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;
    drop(conn);

    let (xp_earned, gold_earned) = if is_workout {
        let minutes = parsed.duration_sec / 60;
        let xp = (minutes * 2).max(25) as i32 + (parsed.calories / 10) as i32 + (steps_actual / 100) as i32;
        let gold = (parsed.calories / 15).max(12) as i32 + (steps_actual / 200) as i32;
        (xp, gold)
    } else {
        (0, 0)
    };

    if is_workout && (xp_earned > 0 || gold_earned > 0) {
        let _ = goblin::add_xp_and_gold(db, xp_earned, gold_earned);
    }

    let activity = GarminActivity {
        id: inserted_activity_id,
        activity_type: parsed.sport.clone(),
        title: parsed.title.clone(),
        start_time: parsed.start_time.clone(),
        duration_sec: parsed.duration_sec as i32,
        distance_meters: parsed.distance_meters,
        calories: parsed.calories as i32,
        avg_hr: parsed.avg_hr as i32,
        max_hr: parsed.max_hr as i32,
        aerobic_training_effect: parsed.aerobic_training_effect.unwrap_or(0.0) as f64,
        anaerobic_training_effect: parsed.anaerobic_training_effect.unwrap_or(0.0) as f64,
    };

    Ok(FitImportResult {
        success: true,
        activity,
        xp_earned: xp_earned as i64,
        gold_earned: gold_earned as i64,
        gps_points_count: parsed.gps_track.len(),
        status_message: if is_workout {
            format!(
                "Successfully synced {} ({} kcal, {} steps)! +{} XP and +{} Gold awarded to your Goblin.",
                parsed.title,
                parsed.calories,
                steps_actual,
                xp_earned,
                gold_earned
            )
        } else {
            format!("Updated Garmin biometrics from {}", parsed.title)
        },
    })
}

fn chrono_or_system_time() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
