use reqwest::{Client, Error, Method, Response};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

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
        body: Option<T>,
    ) -> Result<Response, Error> {
        let url = format!("{}/{}", self.base_url, endpoint);

        let mut request_builder = self.client.request(method.clone(), &url);

        match method {
            Method::GET | Method::DELETE => {
                if let Some(data) = data {
                    // Serialize the data into query parameters
                    let query = serde_urlencoded::to_string(&data).unwrap();
                    request_builder = request_builder.query(&[("data", query)]);
                }
            }
            Method::POST | Method::PUT => {
                if let Some(data) = data {
                    // Serialize the data into JSON body
                    request_builder = request_builder.json(&data);
                }
            }
            _ => {}
        }

        let response = request_builder.send().await?;

        Ok(response)
    }
}

#[derive(Debug)]
pub enum TesterError {
    JsonTypeMismatch {
        endpoint: String,
        client_value: Value,
        actual_value: Value,
    },
}

impl fmt::Display for TesterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TesterError::JsonTypeMismatch {
                endpoint,
                client_value,
                actual_value,
            } => {
                write!(
                    f,
                    "JSON type mismatch at endpoint `{}`.\nClient Value: {:?}\nActual Value: {:?}",
                    endpoint, client_value, actual_value
                )
            }
        }
    }
}

impl std::error::Error for TesterError {}

pub struct Tester {
    client: RequestClient,
    actual: RequestClient,
}

impl Tester {
    pub fn new(test_url: String, server_url: String) -> Self {
        Self {
            client: Client::new(test_url),
            actual: Client::new(server_url),
        }
    }

    pub async fn compare(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<T>,
    ) -> Result<(), Error> {
        let response_client = self.client.request(method.clone(), endpoint, body).await?;
        let response_actual = self.actual.request(method.clone(), endpoint, body).await?;

        let body_client: Value = response_client.json().await?;
        let body_actual: Value = response_actual.json().await?;

        self.compare_json_types(&body_client, &body_actual)
    }

    fn compare_json_types(&self, a: &Value, b: &Value) -> Result<(), TesterError> {
        match (a, b) {
            (Value::Object(map_a), Value::Object(map_b)) => self.compare_json_objects(map_a, map_b),
            (Value::Array(arr_a), Value::Array(arr_b)) => self.compare_json_arrays(arr_a, arr_b),
            (Value::String(_), Value::String(_)) => Ok(()),
            (Value::Number(_), Value::Number(_)) => Ok(()),
            (Value::Bool(_), Value::Bool(_)) => Ok(()),
            (Value::Null, Value::Null) => Ok(()),
            _ => Err(TesterError::JsonTypeMismatch {
                endpoint: endpoint.to_string(),
                client_value: a.clone(),
                actual_value: b.clone(),
            }), // Types do not match
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
                    actual_value: Value::Null, // Value is missing in the other response
                });
            }
        }

        for key in map_b.keys() {
            if !map_a.contains_key(key) {
                return Err(TesterError::JsonTypeMismatch {
                    endpoint: endpoint.to_string(),
                    client_value: Value::Null, // Value is missing in the first response
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
