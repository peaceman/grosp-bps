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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::config;
    use crate::config::AppConfig;
    use crate::http::auth::{validate_jwt, Claims, Error};
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use regex::Regex;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use url::Url;
    use warp::reject::InvalidQuery;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn config() -> std::result::Result<AppConfig, Box<dyn std::error::Error>> {
        Ok(Arc::new(config::Config {
            consul: config::Consul {
                base_url: Url::parse("http://localhost:8500")?,
                update_interval: Default::default(),
            },
            playlist: config::Playlist {
                upstream_base_url: Url::parse("http://localhost")?,
                segment_signing: config::SegmentSigning {
                    key: "".to_string(),
                    duration: Default::default(),
                },
                jwt_validation: config::JwtValidation {
                    secret: "secret".to_string(),
                    stream_name_pattern: Regex::new(r"([^/]+)\.m3u8")?,
                },
            },
            http: config::Http {
                socket: "[::]:23".parse()?,
            },
        }))
    }

    #[tokio::test]
    async fn test_validate_jwt_missing_query_params() -> TestResult {
        let filter = validate_jwt(config()?);

        let result = warp::test::request()
            .path("/meca-foo.m3u8")
            .filter(&filter)
            .await;

        assert!(result.is_err(), "request did not fail");

        let err = result.unwrap_err();
        let dc_err = err.find::<InvalidQuery>();

        assert!(
            dc_err.is_some(),
            "expected invalid query error got {:?}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_jwt_non_jwt_content() -> TestResult {
        let filter = validate_jwt(config()?);

        let result = warp::test::request()
            .path("/meca-foo.m3u8?jwt=foo")
            .filter(&filter)
            .await;

        assert!(result.is_err(), "request did not fail");

        let err = result.unwrap_err();
        let dc_err = err
            .find::<Error>()
            .filter(|e| matches!(e, Error::JWTTokenError));

        assert!(dc_err.is_some(), "expected jwt token error got {:?}", err);

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_jwt_invalid_signature() -> TestResult {
        let filter = validate_jwt(config()?);

        let claims = Claims {
            exp: (SystemTime::now() + Duration::from_secs(600))
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            sn: "stream-name".to_string(),
            ng: "node-group".to_string(),
        };

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret("foo".as_bytes()),
        )?;

        let result = warp::test::request()
            .path(format!("/meca-foo.m3u8?jwt={}", token).as_str())
            .filter(&filter)
            .await;

        assert!(result.is_err(), "request did not fail");

        let err = result.unwrap_err();
        let dc_err = err
            .find::<Error>()
            .filter(|e| matches!(e, Error::JWTTokenError));

        assert!(dc_err.is_some(), "expected jwt token error got {:?}", err);

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_jwt_expired() -> TestResult {
        let filter = validate_jwt(config()?);

        let claims = Claims {
            exp: (SystemTime::now() - Duration::from_secs(600))
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            sn: "stream-name".to_string(),
            ng: "node-group".to_string(),
        };

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )?;

        let result = warp::test::request()
            .path(format!("/meca-foo.m3u8?jwt={}", token).as_str())
            .filter(&filter)
            .await;

        assert!(result.is_err(), "request did not fail");

        let err = result.unwrap_err();
        let dc_err = err
            .find::<Error>()
            .filter(|e| matches!(e, Error::JWTTokenError));

        assert!(dc_err.is_some(), "expected jwt token error got {:?}", err);

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_jwt_stream_name_mismatch() -> TestResult {
        let filter = validate_jwt(config()?);

        let claims = Claims {
            exp: (SystemTime::now() + Duration::from_secs(600))
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            sn: "meca-foo".to_string(),
            ng: "node-group".to_string(),
        };

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )?;

        let result = warp::test::request()
            .path(format!("/nonono.m3u8?jwt={}", token).as_str())
            .filter(&filter)
            .await;

        assert!(result.is_err(), "request did not fail");
        let err = result.unwrap_err();
        let dc_err = err
            .find::<Error>()
            .filter(|e| matches!(e, Error::JWTStreamNameMismatch));

        assert!(
            dc_err.is_some(),
            "expected jwt stream name mismatch error got {:?}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_jwt() -> TestResult {
        let filter = validate_jwt(config()?);

        let claims = Claims {
            exp: (SystemTime::now() + Duration::from_secs(600))
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            sn: "meca-foo".to_string(),
            ng: "node-group".to_string(),
        };

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )?;

        let result = warp::test::request()
            .path(format!("/meca-foo.m3u8?jwt={}", token).as_str())
            .filter(&filter)
            .await;

        assert!(result.is_ok());

        let parsed_claims = result.unwrap();
        assert_eq!(claims, parsed_claims);

        Ok(())
    }
}
