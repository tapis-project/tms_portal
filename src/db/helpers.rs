use anyhow::Result;
use sqlx::{PgPool, PgTransaction};

pub async fn do_in_transaction<F, R>(db_pool: &PgPool, mut dbfn: F) -> Result<R>
where
    F: FnOnce(&mut PgTransaction) -> Result<R>,
{
    let mut tx = db_pool.begin().await?;
    let result = async { dbfn(&mut tx) }.await;

    match result {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
