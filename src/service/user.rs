use axum::Json;

use crate::service::auth::{get_jwt, Claims};
use crate::database::user;
use crate::service::idl::*;

const USERNAME_ALREADY_USED_ERROR_CODE: i32 = 1002;
const USERNAME_ALREADY_USED_ERROR_MESSAGE: &str = "this username is already used, please pick another one.";
const UNEXPECTED_ERROR: ErrorResponse = ErrorResponse {
    code: -1,
    message: "internal error, please contact admin",
};

/// Register a user
pub async fn register(
    Json(req): Json<RegisterRequest>
) -> Result<Json<RegisterResponse>, Json<ErrorResponse>> {
    let username = &req.username;
    let password = &req.password;
    validate_request_fields(username, password)?;

    let res = user::find(username)
        .await
        .map_err(|e| {
            tracing::error!("[user_service] occured unexpected error: {}", e);
            Json(UNEXPECTED_ERROR.clone())
        })?;

    if res.is_some() {
        return Err(Json(ErrorResponse {
            code: USERNAME_ALREADY_USED_ERROR_CODE,
            message: USERNAME_ALREADY_USED_ERROR_MESSAGE
        }));
    }

    user::create(&req.username, &req.password)
        .await
        .map(|_| Json(RegisterResponse { success: true }))
        .map_err(|e| {
            tracing::error!("[user_service] failed to create user: {}, and error: {}", &req.username, e);
            Json(UNEXPECTED_ERROR.clone())
        })
}

/// Login
pub async fn login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    tracing::debug!("[user_service] request: {:?}", &req);
    let username = &req.username;
    let password = &req.password;
    validate_request_fields(username, password)?;

    // Get a user
    let user = user::find(&req.username)
        .await
        .map_err(|e| {
            tracing::error!("[user_service] failed to find user: {}, and error: {}", &req.username, e);
            Json(ErrorResponse {
                code: -1,
                message: "internal error, please contact to admin"
            })
        })?
        .ok_or_else(|| {
            tracing::info!("[user_service] user not found, username: {}", &req.username);
            Json(ErrorResponse {
                code: 1001,
                message: "username or password is not valid"
            })
        })?;

        tracing::debug!("[user_service] user: {:?}", &user);
        tracing::debug!("[user_service] user password: {}, request password: {}", &user.password, &req.password);

        // Check for password correct
        if user.password != req.password {
            tracing::info!("[user_service] password mismatch");
            return Err(Json(ErrorResponse {
                code: 1001,
                message: "username or password is not valid"
            }));
        }

        // generate a JWT
        let token = get_jwt(&Claims { 
            username: username.to_string(),
            exp: 2000000000
        })
        .map_err(|e| {
            tracing::error!("[user_service] generate a JWT token error: {}", e);
            UNEXPECTED_ERROR
        })
        .unwrap();

        Ok(Json(LoginResponse {
            success: true,
            token
        }))
}

fn validate_request_fields(username: &str, password: &str) -> Result<(), Json<ErrorResponse>> {
    if username.is_empty() || password.is_empty() {
        tracing::error!("[user_service] parameter validation failed: username or password cannot be empty");
        return Err(Json(ErrorResponse {
            code: 1001,
            message: "username and password cannot be empty"
        }));
    }
    Ok(())
}