use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use strum_macros::Display;

pub type Result<T> = core::result::Result<T, AuthError>;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr, Display)]
pub enum AuthError {
    LoginFail,
    NoToken,
    WrongTokenFormat,
    TicketDeleteNotFound { id: u64 },
}
#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        println!("Err");

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(self);

        return response;
    }
}

impl AuthError {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            Self::NoToken | Self::WrongTokenFormat => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            Self::TicketDeleteNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}
