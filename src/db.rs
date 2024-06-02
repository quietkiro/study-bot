use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub fn startup(pool: Pool<SqliteConnectionManager>) {
    let conn = pool.get().unwrap();
    // initialize tables if they don't already exist
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS study_log ( 
        user_id INTEGER, 
        start_time INTEGER, 
        end_time INTEGER 
        );
        CREATE TABLE IF NOT EXISTS study_channels (
        channel_id INTEGER,
        guild_id INTEGER
        );
        CREATE TABLE IF NOT EXISTS study_duration (
        user_id INTEGER,
        total_duration INTEGER
        );
        COMMIT;",
    )
    .unwrap();
    // delete logs of any incomplete study sessions
    // because we don't know if users took breaks while the bot
    // was offline
    conn.execute_batch(
        "BEGIN;
        DELETE FROM study_log WHERE end_time = 0;
        COMMIT;",
    )
    .unwrap();
}

pub fn log_start_time(pool: Pool<SqliteConnectionManager>, user_id: u64, start_time: u64) {
    let conn = pool.get().unwrap();
    conn.execute(
        "INSERT INTO study_log (user_id, start_time, end_time) VALUES (?1, ?2, ?3)",
        (user_id, start_time, 0),
    )
    .unwrap();
    match conn.query_row(
        "SELECT total_duration FROM study_duration WHERE user_id = ?1",
        [user_id],
        |row| row.get::<usize, u64>(0),
    ) {
        Err(_) => {
            conn.execute(
                "INSERT INTO study_duration (user_id, total_duration) VALUES (?1, ?2)",
                (user_id, 0u64),
            )
            .unwrap();
        }
        _ => {}
    }
}

pub fn log_end_time(pool: Pool<SqliteConnectionManager>, user_id: u64, end_time: u64) {
    let conn = pool.get().unwrap();
    let start_time = conn
        .query_row(
            "SELECT start_time FROM study_log WHERE end_time = 0 AND user_id = ?",
            params![user_id],
            |row| row.get::<usize, u64>(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE study_log SET end_time = ?1 WHERE user_id = ?2 AND start_time = ?3",
        [end_time, user_id, start_time],
    )
    .unwrap();
    conn.execute(
        "UPDATE study_duration SET total_duration = total_duration + ?1 WHERE user_id = ?2",
        [end_time - start_time, user_id],
    )
    .unwrap();
}

pub fn get_total_study_duration(pool: Pool<SqliteConnectionManager>, user_id: u64) -> u64 {
    let conn = pool.get().unwrap();
    match conn.query_row(
        "SELECT total_duration FROM study_duration WHERE user_id = ?1",
        [user_id],
        |row| row.get(0),
    ) {
        Ok(duration) => duration,
        _ => 0,
    }
}

pub fn get_leaderboard(pool: Pool<SqliteConnectionManager>) -> Vec<(u64, u64)> {
    let conn = pool.get().unwrap();
    let mut stmt = conn
        .prepare("SELECT user_id, total_duration FROM study_duration LIMIT 10")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut result: Vec<(u64, u64)> = vec![];
    while let Ok(Some(row)) = rows.next() {
        let user_id: u64 = row.get(0).unwrap();
        let total_duration: u64 = row.get(1).unwrap();
        result.push((user_id, total_duration));
    }
    result
}

pub fn is_study_channel(pool: Pool<SqliteConnectionManager>, channel_id: u64) -> bool {
    let conn = pool.get().unwrap();
    match conn.query_row(
        "SELECT channel_id FROM study_channels WHERE channel_id = ?1",
        [channel_id],
        |row| row.get::<usize, u64>(0),
    ) {
        Ok(_) => true,
        _ => false,
    }
}

pub fn add_study_channel(pool: Pool<SqliteConnectionManager>, channel_id: u64) {
    let conn = pool.get().unwrap();
    conn.execute(
        "INSERT INTO study_channels (channel_id) VALUES (?1)",
        [channel_id],
    )
    .unwrap();
}

pub fn remove_study_channel(pool: Pool<SqliteConnectionManager>, channel_id: u64) {
    let conn = pool.get().unwrap();
    conn.execute(
        "DELETE FROM study_channels WHERE channel_id = ?1",
        [channel_id],
    )
    .unwrap();
}
