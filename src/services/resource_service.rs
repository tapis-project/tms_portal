use crate::db::keys_dao::db_get_key_by_id;
use crate::db::resource_provider_dao::db_get_resource_providers;
use crate::models::resource_api::GetResourceProviderResponse;
use crate::services::login_service::TmsTokenClaims;
use crate::services::service_error::ServiceError::BadRequest;
use crate::utils::jwt_utils::JwtDecoderBuilder;
use anyhow::Result;
use jsonwebtoken::decode_header;
use sqlx::PgPool;

pub async fn get_resource_providers(
    db_pool: &PgPool,
    token: &String,
) -> Result<GetResourceProviderResponse> {
    let token_header = decode_header(token)?;
    let mut tx = db_pool.begin().await?;
    let key = match token_header.kid {
        Some(kid) => db_get_key_by_id(&mut tx, &kid).await,
        None => return Err(BadRequest(String::from("Unable to find key for jwt")).into()),
    }?;
    tx.commit().await?;

    let tms_token_claims: TmsTokenClaims = JwtDecoderBuilder::builder()
        .public_key(key.jwt_public_key.as_bytes())
        .decode(token)
        .await?;

    let mut tx = db_pool.begin().await?;
    let rps = db_get_resource_providers(&mut tx).await?;
    tx.commit().await?;

    let mut resource_provider_result = GetResourceProviderResponse::new();
    rps.iter().for_each(|rp| {
        resource_provider_result.insert(rp.clone().into());
    });
    Ok(resource_provider_result)
}
