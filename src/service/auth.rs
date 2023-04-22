use axum::{
    extract::FromRequestParts, 
    async_trait, 
    http::request::Parts, 
    response::{
        IntoResponse, 
        Response
    }, 
    TypedHeader, 
    headers::{
        Authorization, 
        authorization::Bearer
    }
};
use jsonwebtoken::{encode, EncodingKey, Header, DecodingKey, Validation, decode};
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};

use lazy_static::lazy_static;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub username: String,
    pub exp: u64
}

lazy_static! {
    static ref JWT_SECRET: String = std::env::var("JWT_SECRET").expect("JWT_SECRET environment dosen't been setted.");
}

/// Generate a custom JWT
pub fn get_jwt(claims: &Claims) -> anyhow::Result<String> {
    encode(
        &Header::default(), 
        claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes())
    )
    .map_err(|e| {
        anyhow::anyhow!(e)
    })
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    // these bounds are required by `async_trait`
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError)?;

        tracing::debug!("[auth_service] Incoming token: {:?}", &bearer);

        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("[auth_service] occured some error: {}", e);
            AuthError
        })?;

        tracing::debug!("[auth_service] token_data: {:?}", &token_data);

        Ok(token_data.claims)
    }
}

/// defines what to do when the request is rejected
pub struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
