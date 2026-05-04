//! API Response Types
//!
//! Standardized response wrapper for all API endpoints.
//! Matches the exact format expected by the control-panel frontend.

use serde::Serialize;

/// Standard API response wrapper.
///
/// All API responses use this format for consistency.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data (only present on success)
    pub data: T,
    /// Error message (only present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response with data.
    ///
    /// # Arguments
    ///
    /// * `data` - Response data
    ///
    /// # Returns
    ///
    /// Success response
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
        }
    }
}
