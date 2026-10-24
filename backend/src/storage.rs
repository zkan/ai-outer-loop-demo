//! SQLite store. Versioned SQL migrations run when the database is opened.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection};

use crate::model::{Todo, TodoUpdate};

const MIGRATIONS: &[&str] = &[include_str!("../migrations/0001_create_todos.sql")];

#[derive(Debug)]
pub struct DbError(String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn db_err(err: impl ToString) -> DbError {
    DbError(err.to_string())
}

#[derive(Clone)]
pub struct Todos {
    conn: Arc<Mutex<Connection>>,
}

impl Todos {
    /// Open `path` (creating it if needed) and apply any pending migrations.
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(db_err)?;
                }
            }
            let conn = Connection::open(&path).map_err(db_err)?;
            conn.busy_timeout(Duration::from_secs(5)).map_err(db_err)?;
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
                .map_err(db_err)?;
            migrate(&conn)?;
            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
            })
        })
        .await
        .expect("db task")
    }

    /// Ordered snapshot: oldest first, then by id.
    pub async fn list(&self) -> Result<Vec<Todo>, DbError> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, text, completed, created_at, updated_at
                     FROM todos
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(db_err)?;
            let rows = stmt
                .query_map([], row_to_todo)
                .map_err(db_err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_err)?;
            Ok(rows)
        })
        .await
    }

    pub async fn create(&self, text: String) -> Result<Todo, DbError> {
        self.with_conn(move |conn| {
            let now = now_rfc3339();
            conn.execute(
                "INSERT INTO todos (text, completed, created_at, updated_at)
                 VALUES (?1, 0, ?2, ?2)",
                params![text, now],
            )
            .map_err(db_err)?;
            let id = conn.last_insert_rowid();
            Ok(load_todo(conn, id)?.expect("inserted row"))
        })
        .await
    }

    pub async fn update(&self, id: &str, updates: TodoUpdate) -> Result<Option<Todo>, DbError> {
        let id = id.to_string();
        self.with_conn(move |conn| {
            let Some(numeric_id) = parse_id(&id) else {
                return Ok(None);
            };
            let Some(current) = load_todo(conn, numeric_id)? else {
                return Ok(None);
            };
            let text = updates.text.unwrap_or(current.text);
            let completed = updates.completed.unwrap_or(current.completed);
            let now = now_rfc3339();
            conn.execute(
                "UPDATE todos
                 SET text = ?1, completed = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![text, completed as i64, now, numeric_id],
            )
            .map_err(db_err)?;
            load_todo(conn, numeric_id)
        })
        .await
    }

    /// Returns `true` when a row matched and was removed.
    pub async fn delete(&self, id: &str) -> Result<bool, DbError> {
        let id = id.to_string();
        self.with_conn(move |conn| {
            let Some(numeric_id) = parse_id(&id) else {
                return Ok(false);
            };
            let changed = conn
                .execute("DELETE FROM todos WHERE id = ?1", [numeric_id])
                .map_err(db_err)?;
            Ok(changed > 0)
        })
        .await
    }

    async fn with_conn<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, DbError> + Send + 'static,
    ) -> Result<T, DbError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let guard = conn.lock().expect("db lock poisoned");
            f(&guard)
        })
        .await
        .expect("db task")
    }
}

fn migrate(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(db_err)?;

    for (index, sql) in MIGRATIONS.iter().enumerate() {
        let version = (index as i64) + 1;
        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        if applied > 0 {
            continue;
        }
        let tx = conn.unchecked_transaction().map_err(db_err)?;
        tx.execute_batch(sql).map_err(db_err)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, now_rfc3339()],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
    }
    Ok(())
}

fn row_to_todo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    let id: i64 = row.get(0)?;
    Ok(Todo {
        id: format_id(id),
        text: row.get(1)?,
        completed: row.get::<_, i64>(2)? != 0,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn load_todo(conn: &Connection, id: i64) -> Result<Option<Todo>, DbError> {
    conn.query_row(
        "SELECT id, text, completed, created_at, updated_at FROM todos WHERE id = ?1",
        [id],
        row_to_todo,
    )
    .map(Some)
    .or_else(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(db_err(other)),
    })
}

fn format_id(id: i64) -> String {
    format!("todo-{id:08x}")
}

fn parse_id(id: &str) -> Option<i64> {
    let hex = id.strip_prefix("todo-")?;
    if hex.len() != 8 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    i64::from_str_radix(hex, 16).ok().filter(|value| *value > 0)
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDb {
        todos: Todos,
        _dir: tempfile::TempDir,
    }

    async fn fresh() -> TestDb {
        let dir = tempfile::tempdir().expect("temp dir");
        let todos = Todos::connect(dir.path().join("todos.db"))
            .await
            .expect("connect");
        TestDb { todos, _dir: dir }
    }

    #[tokio::test]
    async fn list_starts_empty() {
        let db = fresh().await;
        assert!(db.todos.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_then_list_round_trips() {
        let db = fresh().await;
        let a = db.todos.create("first".into()).await.unwrap();
        let b = db.todos.create("second".into()).await.unwrap();

        assert_eq!(a.id, "todo-00000001");
        assert_eq!(a.text, "first");
        assert!(!a.completed);
        assert_eq!(a.updated_at, a.created_at);
        assert_rfc3339(&a.created_at);

        let all = db.todos.list().await.unwrap();
        assert_eq!(all[0].id, a.id);
        assert_eq!(all[1].id, b.id);
    }

    #[tokio::test]
    async fn update_toggle_and_delete() {
        let db = fresh().await;
        let id = db.todos.create("toggle me".into()).await.unwrap().id;
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let updated = db
            .todos
            .update(
                &id,
                TodoUpdate {
                    text: Some("toggled".into()),
                    completed: Some(true),
                },
            )
            .await
            .unwrap();
        let updated = updated.expect("row");
        assert_eq!(updated.text, "toggled");
        assert!(updated.completed);
        assert_ne!(updated.updated_at, updated.created_at);
        assert_rfc3339(&updated.updated_at);

        assert!(db.todos.delete(&id).await.unwrap());
        assert!(db.todos.list().await.unwrap().is_empty());
        assert!(!db.todos.delete(&id).await.unwrap());
    }

    #[tokio::test]
    async fn update_unknown_id_is_noop() {
        let db = fresh().await;
        let out = db
            .todos
            .update(
                "todo-99999999",
                TodoUpdate {
                    text: Some("nope".into()),
                    completed: None,
                },
            )
            .await
            .unwrap();
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn persists_across_reopen() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("todos.db");

        let id = {
            let db = Todos::connect(&path).await.unwrap();
            let todo = db.create("saved".into()).await.unwrap();
            db.update(
                &todo.id,
                TodoUpdate {
                    text: None,
                    completed: Some(true),
                },
            )
            .await
            .unwrap();
            todo.id
        };

        let db = Todos::connect(&path).await.unwrap();
        let all = db.list().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, id);
        assert_eq!(all[0].text, "saved");
        assert!(all[0].completed);

        assert!(db.delete(&id).await.unwrap());
        drop(db);

        let db = Todos::connect(&path).await.unwrap();
        assert!(db.list().await.unwrap().is_empty());
    }

    fn assert_rfc3339(ts: &str) {
        let parsed = chrono::DateTime::parse_from_rfc3339(ts).expect("rfc3339");
        let skew = (Utc::now() - parsed.with_timezone(&Utc))
            .num_seconds()
            .abs();
        assert!(skew < 10, "{ts} is not the current time");
    }
}
