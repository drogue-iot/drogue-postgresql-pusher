use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Error processing JSON path: {0}")]
    Selector(String),
    #[error("Failed processing payload: {0}")]
    PayloadParse(String),
    #[error("Failed converted expected type: {0}")]
    Conversion(String),
    #[error("Error connecting target: {0}")]
    Target(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        let message = format!("{}", self);
        match self {
            ServiceError::Selector { .. } => HttpResponse::NotAcceptable().json(ErrorResponse {
                error: "SelectorError".into(),
                message,
            }),
            ServiceError::PayloadParse { .. } => {
                HttpResponse::NotAcceptable().json(ErrorResponse {
                    error: "PayloadError".into(),
                    message,
                })
            }
            ServiceError::Conversion { .. } => HttpResponse::NotAcceptable().json(ErrorResponse {
                error: "ConversionError".into(),
                message,
            }),
            ServiceError::Target { .. } => HttpResponse::BadGateway().json(ErrorResponse {
                error: "TargetError".into(),
                message,
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
