//! In-memory store. Plain `Vec` + `Mutex` is enough for the MVP; swap this
//! out for a real DB (e.g. SQLx + Postgres) without touching the handlers.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{Todo, TodoCreate, TodoUpdate};

pub struct Todos {
    next_id: u64,
    items: Mutex<Vec<Todo>>,
}

impl Todos {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            items: Mutex::new(Vec::new()),
        }
    }

    /// Ordered snapshot: oldest first, then by id for stability.
    pub fn list(&self) -> Vec<Todo> {
        let mut items = self
            .items
            .lock()
            .expect("storage poisoned");
        items.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        items.clone()
    }

    pub fn get(&self, id: &str) -> Option<Todo> {
        self.items
            .lock()
            .expect("storage poisoned")
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }

    pub fn create(&mut self, text: String) -> Todo {
        let now = now_rfc3339();
        let id = format!("todo-{:08x}", self.next_id);
        self.next_id += 1;

        let todo = Todo {
            id,
            text,
            completed: false,
            created_at: now.clone(),
            updated_at: now,
        };

        self.items.lock().expect("storage poisoned").push(todo.clone());
        todo
    }

    pub fn update(&mut self, id: &str, updates: TodoUpdate) -> Option<Todo> {
        let mut items = self.items.lock().expect("storage poisoned");
        let todo = items.iter_mut().find(|t| t.id == id)?;

        if let Some(text) = updates.text {
            todo.text = text;
        }
        if let Some(completed) = updates.completed {
            todo.completed = completed;
        }
        todo.updated_at = now_rfc3339();
        Some(todo.clone())
    }

    /// Returns `true` when a row matched and was removed.
    pub fn delete(&mut self, id: &str) -> bool {
        let mut items = self.items.lock().expect("storage poisoned");
        let before = items.len();
        items.retain(|t| t.id != id);
        items.len() < before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() -> Todos {
        let t = Todos::new();
        {
            let mut items = t.items.lock().expect("poisoned");
            *items = vec![];
        }
        t
    }

    #[test]
    fn list_starts_empty() {
        let mut s = reset();
        assert!(s.list().is_empty());
    }

    #[test]
    fn create_then_list_round_trips() {
        let mut s = reset();
        let a = s.create("first".into());
        let b = s.create("second".into());

        assert_eq!(a.id, "todo-00000001");
        assert_eq!(a.text, "first");
        assert!(!a.completed);
        assert!(a.created_at.starts_with('2'));
        assert_eq!(a.updated_at, a.created_at);

        let mut all = s.list();
        assert_eq!(all[0].id, a.id);
        assert_eq!(all[1].id, b.id);
    }

    #[test]
    fn update_toggle_and_delete() {
        let mut s = reset();
        let id = s.create("toggle me".into()).id;

        let updated = s.update(&id, TodoUpdate {
            text: Some("toggled".into()),
            completed: Some(true),
        });
        assert_eq!(updated.as_ref().unwrap().text, "toggled");
        assert!(updated.as_ref().unwrap().completed);

        assert!(s.delete(&id));
        assert!(s.list().is_empty());
        assert!(!s.delete(&id)); // gone already
    }

    #[test]
    fn update_unknown_id_is_noop() {
        let mut s = reset();
        let out = s.update("todo-99999999", TodoUpdate {
            text: Some("nope".into()),
            completed: None,
        });
        assert!(out.is_none());
    }
}

fn now_rfc3339() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    let secs = dur.as_secs();
    let nsecs = dur.subsec_nanos();

    let (year, month, day) = unix_to_ymd(secs);
    // RFC 3339 / ISO 8601, UTC.
    format!(
        "{:04}-{:02}-{:02}T00:00:00.{:09}Z",
        year, month, day, nsecs
    )
}

/// Convert Unix seconds to (year, month, day); day 0 == 1970-01-01.
fn unix_to_ymd(secs: u64) -> (u64, u32, u32) {
    let days = secs / 86400;
    let z = days as i64; // day 0 == 1970-01-01

    // Howard Hinnant's `civil_from_days` is defined on a proleptic
    // Gregorian epoch where day 0 == 0000-03-01, so shift.
    const DAYS_EPOCH0: i64 = 719468; // days from 0000-03-01 to 1970-01-01
    let jd = z + DAYS_EPOCH0;

    let era = if jd >= 0 {
        jd / 146097
    } else {
        (jd - 146096) / 146097
    };
    let doe = jd - era * 146097; // day of era, [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 400)
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365)
    let mp = (5 * doy + 2) / 153; // month piece, [0, 11]
    let dd = doy - (153 * mp + 2) / 5 + 1; // day of month, [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month, [1, 12]
    let yy = if m <= 2 { y + 1 } else { y }; // corrected year

    (yy as u64, m as u32, dd as u32)
}
