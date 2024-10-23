use reqwest::{Client, Error, Method, Response};
use serde_json::Value;
use std::collections::HashMap;

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
        query_params: Option<HashMap<&str, &str>>,
        json_body: Option<Value>,
    ) -> Result<Response, Error> {
        let url = format!("{}/{}", self.base_url, endpoint);

        let mut request_builder = self.client.request(method.clone(), &url);

        if method == Method::GET || method == Method::DELETE {
            if let Some(params) = query_params {
                request_builder = request_builder.query(&params);
            }
        }

        if method == Method::POST || method == Method::PUT {
            if let Some(body) = json_body {
                request_builder = request_builder.json(&body);
            }
        }

        let response = request_builder.send().await?;

        Ok(response)
    }
}