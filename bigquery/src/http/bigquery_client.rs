use crate::http::error::{Error, ErrorWrapper};

use google_cloud_token::TokenSource;
use reqwest::{Client, RequestBuilder, Response};

use std::sync::Arc;

pub const SCOPES: [&str; 7] = [
    "https://www.googleapis.com/auth/bigquery",
    "https://www.googleapis.com/auth/bigquery.insertdata",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloud-platform.read-only",
    "https://www.googleapis.com/auth/devstorage.full_control",
    "https://www.googleapis.com/auth/devstorage.read_only",
    "https://www.googleapis.com/auth/devstorage.read_write",
];

#[derive(Clone)]
pub struct BigqueryClient {
    ts: Arc<dyn TokenSource>,
    endpoint: String,
    http: Client,
}

impl BigqueryClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str, http: Client) -> Self {
        Self {
            ts,
            endpoint: format!("{endpoint}/bigquery/v2"),
            http,
        }
    }

    pub(crate) fn endpoint(&self) -> &str {
        self.endpoint.as_str()
    }

    pub(crate) fn http(&self) -> &Client {
        &self.http
    }

    async fn with_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, Error> {
        let token = self.ts.token().await.map_err(Error::TokenSource)?;
        Ok(builder
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-bigquery")
            .header(reqwest::header::AUTHORIZATION, token))
    }

    pub async fn send<T>(&self, builder: RequestBuilder) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        let response = Self::check_response_status(response).await?;
        //TODO json
        let text = response.text().await?;
        tracing::info!("{}", text);
        Ok(serde_json::from_str(text.as_str()).unwrap())
    }

    pub async fn send_get_empty(&self, builder: RequestBuilder) -> Result<(), Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        Self::check_response_status(response).await?;
        Ok(())
    }

    /// Checks whether an HTTP response is successful and returns it, or returns an error.
    async fn check_response_status(response: Response) -> Result<Response, Error> {
        // Check the status code, returning the response if it is not an error.
        let error = match response.error_for_status_ref() {
            Ok(_) => return Ok(response),
            Err(error) => error,
        };

        // try to extract a response error, falling back to the status error if it can not be parsed.
        Err(response
            .json::<ErrorWrapper>()
            .await
            .map(|wrapper| Error::Response(wrapper.error))
            .unwrap_or(Error::HttpClient(error)))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::http::bigquery_client::{BigqueryClient, SCOPES};
    use crate::http::table::{TableFieldMode, TableFieldSchema, TableFieldType, TableSchema};
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;
    use time::OffsetDateTime;

    pub async fn create_client() -> (BigqueryClient, String) {
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
            sub: None,
        })
        .await
        .unwrap();
        let cred = tsp.source_credentials.clone();
        let ts = tsp.token_source();
        let client = BigqueryClient::new(ts, "https://bigquery.googleapis.com", reqwest::Client::new());
        (client, cred.unwrap().project_id.unwrap())
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TestDataStruct {
        pub f1: bool,
        pub f2: Vec<i64>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TestData {
        pub col_string: Option<String>,
        pub col_number: Option<i32>,
        pub col_number_array: Vec<i32>,
        #[serde(default, with = "time::serde::rfc3339::option")]
        pub col_timestamp: Option<OffsetDateTime>,
        pub col_json: Option<String>,
        pub col_json_array: Vec<String>,
        pub col_struct: Option<TestDataStruct>,
        pub col_struct_array: Vec<TestDataStruct>,
    }

    pub fn create_table_schema() -> TableSchema {
        TableSchema {
            fields: vec![
                TableFieldSchema {
                    name: "col_string".to_string(),
                    data_type: TableFieldType::String,
                    max_length: Some(32),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_number".to_string(),
                    data_type: TableFieldType::Numeric,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_number_array".to_string(),
                    data_type: TableFieldType::Numeric,
                    mode: Some(TableFieldMode::Repeated),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_timestamp".to_string(),
                    data_type: TableFieldType::Timestamp,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_json".to_string(),
                    data_type: TableFieldType::Json,
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_json_array".to_string(),
                    data_type: TableFieldType::Json,
                    mode: Some(TableFieldMode::Repeated),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_struct".to_string(),
                    data_type: TableFieldType::Struct,
                    fields: Some(vec![
                        TableFieldSchema {
                            name: "f1".to_string(),
                            data_type: TableFieldType::Bool,
                            ..Default::default()
                        },
                        TableFieldSchema {
                            name: "f2".to_string(),
                            data_type: TableFieldType::Int64,
                            mode: Some(TableFieldMode::Repeated),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col_struct_array".to_string(),
                    data_type: TableFieldType::Struct,
                    fields: Some(vec![
                        TableFieldSchema {
                            name: "f1".to_string(),
                            data_type: TableFieldType::Bool,
                            ..Default::default()
                        },
                        TableFieldSchema {
                            name: "f2".to_string(),
                            data_type: TableFieldType::Int64,
                            mode: Some(TableFieldMode::Repeated),
                            ..Default::default()
                        },
                    ]),
                    mode: Some(TableFieldMode::Repeated),
                    ..Default::default()
                },
            ],
        }
    }
}
