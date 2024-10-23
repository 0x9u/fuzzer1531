use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to serialize query parameters: {0}")]
    UrlEncodeError(#[from] serde_urlencoded::ser::Error),

    #[error("Tester error: {0}")]
    TesterError(#[from] TesterError),
}

#[derive(Debug, Error)]
pub enum TesterError {
    #[error("JSON type mismatch at endpoint `{endpoint}`.\nClient Value: {client_value:?}\nActual Value: {actual_value:?}")]
    JsonTypeMismatch {
        endpoint: String,
        client_value: Value,
        actual_value: Value,
    },
}

pub struct Tester {
    client: Arc<RequestClient>,
    actual: Arc<RequestClient>,
}

impl Tester {
    pub fn new(test_url: String, server_url: String) -> Self {
        Self {
            client: Arc::new(RequestClient::new(test_url)),
            actual: Arc::new(RequestClient::new(server_url)),
        }
    }

    pub async fn compare(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<Value>,
    ) -> Result<(), RequestError> {
        let response_client = self.client.request(method.clone(), endpoint, body.clone()).await?;
        let response_actual = self.actual.request(method, endpoint, body).await?;

        let body_client: Value = response_client.json().await?;
        let body_actual: Value = response_actual.json().await?;

        self.compare_json_types(&body_client, &body_actual, endpoint).map_err(RequestError::from)
    }

    fn compare_json_types(
        &self,
        a: &Value,
        b: &Value,
        endpoint: &str,
    ) -> Result<(), TesterError> {
        match (a, b) {
            (Value::Object(map_a), Value::Object(map_b)) => {
                self.compare_json_objects(map_a, map_b, endpoint)
            }
            (Value::Array(arr_a), Value::Array(arr_b)) => {
                self.compare_json_arrays(arr_a, arr_b, endpoint)
            }
            (Value::String(_), Value::String(_)) => Ok(()),
            (Value::Number(_), Value::Number(_)) => Ok(()),
            (Value::Bool(_), Value::Bool(_)) => Ok(()),
            (Value::Null, Value::Null) => Ok(()),
            _ => Err(TesterError::JsonTypeMismatch {
                endpoint: endpoint.to_string(),
                client_value: a.clone(),
                actual_value: b.clone(),
            }),
        }
    }

    fn compare_json_objects(
        &self,
        map_a: &serde_json::Map<String, Value>,
        map_b: &serde_json::Map<String, Value>,
        endpoint: &str,
    ) -> Result<(), TesterError> {
        for (key, value_a) in map_a {
            if let Some(value_b) = map_b.get(key) {
                self.compare_json_types(value_a, value_b, endpoint)?;
            } else {
                return Err(TesterError::JsonTypeMismatch {
                    endpoint: endpoint.to_string(),
                    client_value: value_a.clone(),
                    actual_value: Value::Null,
                });
            }
        }

        for key in map_b.keys() {
            if !map_a.contains_key(key) {
                return Err(TesterError::JsonTypeMismatch {
                    endpoint: endpoint.to_string(),
                    client_value: Value::Null,
                    actual_value: map_b.get(key).unwrap().clone(),
                });
            }
        }

        Ok(())
    }

    fn compare_json_arrays(
        &self,
        arr_a: &[Value],
        arr_b: &[Value],
        endpoint: &str,
    ) -> Result<(), TesterError> {
        if arr_a.len() != arr_b.len() {
            return Err(TesterError::JsonTypeMismatch {
                endpoint: endpoint.to_string(),
                client_value: Value::Array(arr_a.to_vec()),
                actual_value: Value::Array(arr_b.to_vec()),
            });
        }

        for (elem_a, elem_b) in arr_a.iter().zip(arr_b.iter()) {
            self.compare_json_types(elem_a, elem_b, endpoint)?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct RequestClient {
    base_url: String,
    client: Client,
}

impl RequestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let mut request_builder = self.client.request(method, &url);

        if let Some(data) = body {
            request_builder = request_builder.json(&data);
        }

        let response = request_builder.send().await?;
        Ok(response)
    }
}