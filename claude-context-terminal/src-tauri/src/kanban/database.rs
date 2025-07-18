use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::PathBuf;
use dirs::home_dir;
use anyhow::Result;

static mut DB_POOL: Option<Pool<Sqlite>> = None;

pub async fn init_db() -> Result<()> {
    let db_path = get_db_path()?;
    
    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite:{}", db_path.display());
    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    unsafe {
        DB_POOL = Some(pool);
    }

    Ok(())
}

pub fn get_pool() -> &'static Pool<Sqlite> {
    unsafe {
        DB_POOL.as_ref().expect("Database not initialized")
    }
}

fn get_db_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let db_dir = home.join(".ClaudeContext");
    Ok(db_dir.join("tasks.db"))
}