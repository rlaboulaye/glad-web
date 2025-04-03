static DB: std::sync::OnceLock<sqlx::SqlitePool> = std::sync::OnceLock::new();

async fn create_pool() -> sqlx::SqlitePool {
    let database_url = std::env::var("DATABASE_URL").expect("no database url specify");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(4)
        .connect(database_url.as_str())
        .await
        .expect("could not connect to database_url");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    pool
}

pub async fn init_db() -> Result<(), sqlx::Pool<sqlx::Sqlite>> {
    DB.set(create_pool().await)
}

pub fn get_db<'a>() -> &'a sqlx::SqlitePool {
    DB.get().expect("database unitialized")
}
