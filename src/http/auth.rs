use crate::config::AppConfig;
use crate::http::WebResult;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error as ThisError;
use warp::{reject, Filter, Rejection};

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("jwt was not valid")]
    JWTTokenError,
    #[error("the stream name from doesn't match")]
    JWTStreamNameMismatch,
}

impl warp::reject::Reject for Error {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    // Expiry epoch
    exp: u64,
    // Stream name
    sn: String,
    // NodeGroup name
    ng: String,
}

impl Claims {
    pub fn node_group(&self) -> &str {
        self.ng.as_str()
    }
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
        .and(warp::path::peek())
        .and_then(validate)
}

async fn validate(
    config: AppConfig,
    params: AuthQueryParams,
    path: warp::path::Peek,
) -> WebResult<Claims> {
    let token_data: TokenData<Claims> = jsonwebtoken::decode(
        &params.jwt,
        &DecodingKey::from_secret(config.playlist.jwt_validation.secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|_| reject::custom(Error::JWTTokenError))
    .and_then(|td| {
        validate_stream_name(&td.claims, &config, &path)
            .map_err(reject::custom)
            .map(|_| td)
    })?;

    Ok(token_data.claims)
}

fn validate_stream_name(
    claims: &Claims,
    config: &AppConfig,
    path: &warp::path::Peek,
) -> Result<(), Error> {
    let re = &config.playlist.jwt_validation.stream_name_pattern;

    re.captures(path.as_str())
        .and_then(|captures| captures.get(1))
        .and_then(|m| match m.as_str() == claims.sn {
            true => Some(()),
            false => None,
        })
        .ok_or(Error::JWTStreamNameMismatch)
}
