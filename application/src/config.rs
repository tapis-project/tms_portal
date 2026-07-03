use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_db(connection_url: &String) -> PgPool {
    // unwrap - panic and exist if we can't conenct to db
    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await
        .unwrap()
}
