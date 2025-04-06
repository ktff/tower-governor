use http::{HeaderMap, Response, StatusCode};
use std::mem;
use thiserror::Error;
use tonic::{Code, Status};

use crate::Body;

/// The error type returned by tower-governor.
#[derive(Debug, Error, Clone)]
pub enum GovernorError {
    #[error("Too Many Requests! Wait for {wait_time}s")]
    TooManyRequests {
        wait_time: u64,
        headers: Option<HeaderMap>,
    },
    #[error("Unable to extract key!")]
    UnableToExtractKey,
    #[error("Other Error")]
    /// Used for custom key extractors to return their own errors
    Other {
        code: StatusCode,
        msg: Option<String>,
        headers: Option<HeaderMap>,
    },
}

impl GovernorError {
    pub(crate) fn as_response(&mut self) -> Response<Body> {
        let status = match mem::replace(self, Self::UnableToExtractKey) {
            GovernorError::TooManyRequests { wait_time, headers } => {
                let status = Status::new(
                    Code::Unavailable,
                    format!("Too Many Requests! Wait for {}s", wait_time),
                );
                if let Some(mut headers) = headers {
                    let _ = status.add_header(&mut headers);
                }

                status
            }
            GovernorError::UnableToExtractKey => {
                Status::new(Code::Internal, "Unable To Extract Key!".to_string())
            }
            GovernorError::Other {
                msg,
                code: _,
                headers,
            } => {
                let status = Status::new(Code::Internal, msg.unwrap_or_default());

                // parts.status = code;
                if let Some(mut headers) = headers {
                    let _ = status.add_header(&mut headers);
                }
                status
            }
        };
        status.into_http()
    }
}
