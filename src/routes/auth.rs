use crate::db::config_dao::get_state_cookie_path;
use crate::db::idp_dao::{get_idp_by_id, get_idps};
use crate::models::api::{Entity, TmsResponse, TokenResponse};
use crate::models::oauth2::IdpResponse;
use crate::models::oauth2::{
    AuthCodeQueryParams, AuthorizationCodeResponse, AuthorizeByIdpRequest,
};
use crate::models::service_error::AppError;
use crate::models::service_error::ServiceError::{BadRequest, Internal, NotFound, Unauthorized};
use crate::models::tms_internal::OAuthState;
use crate::services::oauth_service::{
    decode_access_token, decode_state, encode_state, exchange_code_for_token, make_auth_token,
};
//use crate::models::tms_internal::TmsServiceError::{BadRequest, NotFoundError, Unauthorized};
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use axum::{debug_handler, extract::Query, routing::get, Form, Router};
use axum_extra::extract::PrivateCookieJar;
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/callback", get(get_callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        // .route("/oauth2/test", get(testit))
        .route("/oauth2/authorize", get(get_authorize_handler))
}
// pub async fn thing<'a>(tx: &mut PgTransaction<'a>) -> Result<HashSet<IdpResponse>> {
//     let idps = get_idps(tx).await?;
//     let mut idp_result: HashSet<IdpResponse> = HashSet::new();
//     idps.iter().for_each(|idp| {
//         idp_result.insert(idp.clone().into());
//     });
//     Ok(idp_result)
// }

#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> Result<TmsResponse<HashSet<IdpResponse>>, AppError> {
    // //    let mut theTx = app_state.db_pool.begin().await?;
    // // let state = Rc::new(app_state);
    //
    // // let idps = do_in_transaction(&state.db_pool, |tx| async {
    // //     let r = get_idps(tx).await?;
    // //     Ok(r.clone())
    // // })
    // // .await?;
    // // let idp_result = do_in_transaction(&app_state.db_pool, thing).await?;
    // let idps: HashSet<Idp> =
    //     do_in_transaction(&app_state.db_pool, |tx: &mut PgTransaction| async {
    //         get_idps(tx).await
    //         // Err(anyhow!("exiting".to_string()))
    //     })
    //     .await?;
    // let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    // idps.iter().for_each(|idp| {
    //     idp_result.insert(idp.clone().into());
    // });
    //
    let mut tx = app_state.db_pool.begin().await?;
    let idps = match get_idps(&mut tx).await {
        Ok(idps) => {
            tx.commit().await?;
            idps
        }
        Err(error) => {
            tx.rollback().await?;
            return Err(Internal(error.to_string()).into());
        }
    };

    let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    idps.iter().for_each(|idp| {
        idp_result.insert(idp.clone().into());
    });

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(Entity::Success(idp_result))
        .build())

    // // Ok(TmsResponse::builder(StatusCode::OK)
    // //     .entity(Entity::Success(idp_result))
    // //     .build())
    // let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    // let mut tx = app_state.db_pool.begin().await.unwrap();
    // match get_idps(&mut tx).await {
    //     Ok(idps) => {
    //         tx.commit().await.unwrap();
    //         idps.iter().for_each(|idp| {
    //             idp_result.insert(idp.clone().into());
    //         });
    //         Ok(TmsResponse::builder(StatusCode::OK)
    //             .entity(Entity::Success(idp_result))
    //             .build())
    //     }
    //
    //     Err(error) => {
    //         tx.rollback().await.unwrap();
    //         Err(Internal(error.to_string()).into())
    //         // TmsResponse::builder(StatusCode::INTERNAL_SERVER_ERROR)
    //         //     .entity_from(error)
    //         //     .build()
    //     }
    // }
}

#[debug_handler]
pub async fn get_callback_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<TmsResponse<TokenResponse>, AppError> {
    // get and decode state
    let state_string = match &query_params.state {
        Some(state_string) => state_string,
        None => return Err(BadRequest("Missing query parameter: State".to_string()).into()),
    };

    let cookie_state = match jar.get(&get_state_cookie_path()) {
        Some(idp_cookie) if idp_cookie.value().eq(state_string) => idp_cookie.value().to_owned(),
        _ => return Err(Unauthorized("No state cookies were found".to_string()).into()),
    };
    dbg!(&cookie_state);

    let state = match decode_state(state_string).await {
        Ok(state) => state,
        Err(error) => return Err(Unauthorized(error.to_string()).into()),
    };
    dbg!(&state);

    let mut tx = app_state.db_pool.begin().await?;
    let idp = match get_idp_by_id(&mut tx, &state.idp_id).await {
        Ok(idp) => {
            tx.commit().await?;
            idp
        }
        Err(_) => {
            tx.rollback().await?;
            return Err(NotFound("Idp was not found".to_string()).into());
        }
    };

    let token: AuthorizationCodeResponse =
        match exchange_code_for_token(&idp, &query_params.code).await {
            Ok(token) => token,
            Err(error) => return Err(Internal(error.to_string()).into()),
        };

    dbg!(&token);
    let claims: HashMap<String, Value> = match decode_access_token(&idp, &token.id_token).await {
        Ok(claims) => claims,
        Err(error) => return Err(Internal(error.to_string()).into()),
    };

    match make_auth_token(claims).await {
        Ok(token) => Ok(TmsResponse::builder(StatusCode::OK)
            .entity(Entity::Success(TokenResponse { token }))
            .build()),
        Err(error) => Err(Internal(error.to_string()).into()),
    }
}

#[debug_handler]
pub async fn get_authorize_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(PrivateCookieJar, TmsResponse<()>), AppError> {
    let mut tx = app_state.db_pool.begin().await?;
    let idp = get_idp_by_id(&mut tx, &form_data.idp_id).await;

    match idp {
        Ok(idp) => {
            tx.commit().await?;
            let oauth_state = OAuthState {
                idp_id: form_data.idp_id.clone(),
                exp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 300000,
            };

            let encoded_state = match encode_state(oauth_state).await {
                Ok(state_string) => state_string,
                Err(error) => return Err(Internal(error.to_string()).into()),
            };

            // TODO:  make a real nonce
            let location = Url::parse_with_params(
                &idp.identity_redirect_url,
                [
                    ("response_type", "code"),
                    ("client_id", &idp.client_id),
                    ("redirect_uri", "http://localhost:8080/oauth2/callback"),
                    ("scope", &idp.scope),
                    ("state", &encoded_state),
                    ("nonce", "TODO: Add a real nonce"),
                ],
            )
            .unwrap();

            let updated_jar = jar.add((get_state_cookie_path(), encoded_state));

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            Ok((
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ))
        }

        Err(error) => {
            tx.rollback().await?;
            Err(BadRequest(error.to_string()).into())
        }
    }
}

// pub async fn testit() -> anyhow::Result<TmsResponse<String>, AppError> {
//     let resp = isit().await.context("IISit failed:")?;
//     //    let resp = isit().await.with_context(|| "IISit failed:")?;
//     let resp = isit().await?;
//     Ok(TmsResponse::builder(StatusCode::OK)
//         .entity(Success(resp))
//         .build())
// }
//
// pub async fn isit() -> Result<String> {
//     let millis = SystemTime::now()
//         .duration_since(SystemTime::UNIX_EPOCH)?
//         .as_millis();
//     if millis.is_multiple_of(2) {
//         Ok("It's OK".to_string())
//     } else if millis.is_multiple_of(3) {
//         let err = Internal("It's NOT OK".to_string());
//         Err(Internal("It's NOT OK".to_string()).into())
//     } else {
//         let err = Internal("It's NOT OK".to_string());
//         Err(bail!("It's REEALLY NOT OK".to_string()))
//     }
// }
