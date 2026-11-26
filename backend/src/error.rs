use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::{dataflows, models};

pub(crate) struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) message: String,
}

impl From<dataflows::DataflowError> for ApiError {
    fn from(error: dataflows::DataflowError) -> Self {
        match error {
            dataflows::DataflowError::NotFound(message) => Self {
                status: StatusCode::NOT_FOUND,
                message,
            },
            dataflows::DataflowError::Invalid(message) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message,
            },
            dataflows::DataflowError::Io(message) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(models::ApiError {
                error: self.message,
            }),
        )
            .into_response()
    }
}
