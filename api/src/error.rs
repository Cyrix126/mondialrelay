use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;
use deadpool_diesel::{InteractError, PoolError};
use thiserror::Error;

#[derive(Debug, Error, ErrorStatus)]
pub enum AppError {
    #[error("API returned an error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    PgError(#[from] PoolError),
    #[error("API returned an error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    DeadPoolError(#[from] InteractError),
    #[error("API returned an error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    DieselError(#[from] diesel::result::Error),
    #[error("Misconfigured cover API on server side")]
    #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
    Conf,
    #[error("Request xml content is not valid from the mondial relay schema: {0}")]
    #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
    Xml(String),
    #[error("Response xml from mondialrelay does not contains the label: {0}")]
    #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
    NoLabel(String),
    /// The API response status code is an error.
    #[error(transparent)]
    #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
    ReqwestError(#[from] reqwest::Error),
    #[error("The order does not exist.")]
    #[status(axum::http::StatusCode::BAD_REQUEST)]
    OrderNotFound,
    #[error("The address is incorrect: {0}")]
    #[status(axum::http::StatusCode::BAD_REQUEST)]
    BadAddress(String),
}
