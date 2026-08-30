use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventItem {
    pub id: String,
    pub date: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: String,
    pub end_time: String,
    pub is_all_day: bool,
    pub calendar_name: String,
    pub event_color: String,
}

pub struct CalendarService;

impl CalendarService {
    pub fn save_calendar_events(
        conn: &Connection,
        events: &[CalendarEventItem],
    ) -> Result<usize, String> {
        let mut count = 0;
        let mut stmt = conn
            .prepare(
                "INSERT INTO calendar_events (
                    id, date, title, description, location, start_time, end_time, is_all_day, calendar_name, event_color, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
                ON CONFLICT(id) DO UPDATE SET
                    date = excluded.date,
                    title = excluded.title,
                    description = excluded.description,
                    location = excluded.location,
                    start_time = excluded.start_time,
                    end_time = excluded.end_time,
                    is_all_day = excluded.is_all_day,
                    calendar_name = excluded.calendar_name,
                    event_color = excluded.event_color,
                    updated_at = CURRENT_TIMESTAMP",
            )
            .map_err(|e| e.to_string())?;

        for ev in events {
            stmt.execute((
                &ev.id,
                &ev.date,
                &ev.title,
                &ev.description,
                &ev.location,
                &ev.start_time,
                &ev.end_time,
                ev.is_all_day,
                &ev.calendar_name,
                &ev.event_color,
            ))
            .map_err(|e| e.to_string())?;
            count += 1;
        }

        Ok(count)
    }

    pub fn get_events_for_date(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<CalendarEventItem>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, title, description, location, start_time, end_time, is_all_day, calendar_name, event_color
                 FROM calendar_events
                 WHERE date = ?1
                 ORDER BY is_all_day DESC, start_time ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([date], |row| {
                Ok(CalendarEventItem {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    location: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    start_time: row.get(5)?,
                    end_time: row.get(6)?,
                    is_all_day: row.get(7)?,
                    calendar_name: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "Calendar".to_string()),
                    event_color: row.get::<_, Option<String>>(9)?.unwrap_or_else(|| "#10B981".to_string()),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r.map_err(|e| e.to_string())?);
        }
        Ok(events)
    }

    pub fn get_events_for_range(
        conn: &Connection,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<CalendarEventItem>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, date, title, description, location, start_time, end_time, is_all_day, calendar_name, event_color
                 FROM calendar_events
                 WHERE date >= ?1 AND date <= ?2
                 ORDER BY date ASC, is_all_day DESC, start_time ASC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([start_date, end_date], |row| {
                Ok(CalendarEventItem {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    location: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    start_time: row.get(5)?,
                    end_time: row.get(6)?,
                    is_all_day: row.get(7)?,
                    calendar_name: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "Calendar".to_string()),
                    event_color: row.get::<_, Option<String>>(9)?.unwrap_or_else(|| "#10B981".to_string()),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r.map_err(|e| e.to_string())?);
        }
        Ok(events)
    }
}
