use crate::config::AppConfig;
use crate::http::WebResult;
use futures::future;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error as ThisError;
use warp::{reject, Filter, Rejection};

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("jwt was not valid")]
    JWTTokenError,
}

impl warp::reject::Reject for Error {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: u64,
    sn: String,
    ng: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthQueryParams {
    jwt: String,
}

pub fn validate_jwt(
    config: AppConfig,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    let settings = warp::any().map(move || Arc::clone(&config));

    settings
        .and(warp::query::<AuthQueryParams>())
        .and_then(validate)
}

async fn validate(config: AppConfig, params: AuthQueryParams) -> WebResult<Claims> {
    let token_data: TokenData<Claims> = jsonwebtoken::decode(
        &params.jwt,
        &DecodingKey::from_secret(config.playlist.jwt_validation.secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|_| reject::custom(Error::JWTTokenError))?;

    Ok(token_data.claims)
}
