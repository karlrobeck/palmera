use axum::{http::StatusCode, response::IntoResponse};

pub enum AuthError {
    InvalidCredentials(String),
    Sqlx(sqlx::Error),
    Jwt(jwt::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidCredentials(err) => (StatusCode::FORBIDDEN, err).into_response(),
            Self::Sqlx(err) => match err {
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
            },
            Self::Jwt(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}
