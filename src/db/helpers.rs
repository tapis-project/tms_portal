use crate::AppState;
use anyhow::Result;
use sqlx::PgTransaction;

pub async fn do_in_transaction<F, R>(state: AppState, dbfn: F) -> Result<R>
where
    F: FnOnce(&mut PgTransaction) -> Result<R>,
{
    let mut tx = state.db_pool.begin().await?;
    match dbfn(&mut tx) {
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
