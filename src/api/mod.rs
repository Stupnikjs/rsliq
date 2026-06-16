use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub mod positions;
pub mod queries;
pub  mod types;
pub mod number; 


const MORPHO_GRAPHQL_URL: &str = "https://api.morpho.org/graphql";


pub use positions::fetch_all_positions;


pub struct HttpClient {
    url: String,
    client: Client,
}

#[derive(Serialize)]
struct QueryBody<'a> {
    query: &'a str,
}

#[derive(Deserialize)]
struct Envelope {
    data: Option<Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize, Debug)]
pub struct GraphQLError {
    pub message: String,
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "graphql: {}", self.message)
    }
}
impl std::error::Error for GraphQLError {}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            url: MORPHO_GRAPHQL_URL.to_string(),
            client: Client::new(),
        }
    }

    pub async fn query<T: DeserializeOwned>(&self, query: &str) -> anyhow::Result<T> {
        let resp = self
            .client
            .post(&self.url)
            .json(&QueryBody { query })
            .send()
            .await?;

        let envelope: Envelope = resp.json().await?;

        // err check
        if let Some(mut errors) = envelope.errors {
            if let Some(first) = errors.pop() {
                return Err(first.into());
            }
        }

        let data = envelope
            .data
            .ok_or_else(|| anyhow::anyhow!("empty data field"))?;
        Ok(serde_json::from_value(data)?)
    }
}