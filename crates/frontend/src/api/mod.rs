use gloo::net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// API error
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// API client for EASM backend
pub struct ApiClient {
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            token: None,
        }
    }

    /// Set the auth token
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Clear the auth token (for logout)
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Get the URL for an endpoint
    fn get_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }

    /// Execute a GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, ApiError> {
        let url = self.get_url(endpoint);

        let mut request = Request::get(&url);

        // Add auth header if token is present
        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::process_response(response).await
    }

    /// Execute a POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.get_url(endpoint);

        let mut request = Request::post(&url);

        // Add auth header if token is present
        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        // Add JSON body
        let response = request
            .json(body)
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::process_response(response).await
    }

    /// Execute a PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.get_url(endpoint);

        let mut request = Request::put(&url);

        // Add auth header if token is present
        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        // Add JSON body
        let response = request
            .json(body)
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::process_response(response).await
    }

    /// Execute a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, ApiError> {
        let url = self.get_url(endpoint);

        let mut request = Request::delete(&url);

        // Add auth header if token is present
        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::process_response(response).await
    }

    /// Process the API response
    async fn process_response<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
        match response.status() {
            200 | 201 => response
                .json::<T>()
                .await
                .map_err(|e| ApiError::DeserializationError(e.to_string())),
            401 | 403 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::AuthError(text))
            }
            404 => Err(ApiError::NotFound),
            400 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::BadRequest(text))
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(text))
            }
        }
    }
}
