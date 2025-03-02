use std::sync::Arc;

use reqwest::Response;
use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use token_source::TokenSource;

use crate::http::error::{Error, ErrorWrapper};

pub const SCOPES: [&str; 7] = [
    "https://www.googleapis.com/auth/bigquery",
    "https://www.googleapis.com/auth/bigquery.insertdata",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloud-platform.read-only",
    "https://www.googleapis.com/auth/devstorage.full_control",
    "https://www.googleapis.com/auth/devstorage.read_only",
    "https://www.googleapis.com/auth/devstorage.read_write",
];

#[derive(Debug, Clone)]
pub struct BigqueryClient {
    ts: Arc<dyn TokenSource>,
    endpoint: String,
    http: Client,
    debug: bool,
}

impl BigqueryClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str, http: Client, debug: bool) -> Self {
        Self {
            ts,
            endpoint: format!("{endpoint}/bigquery/v2"),
            http,
            debug,
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
        if self.debug {
            let text = response.text().await?;
            tracing::info!("{}", text);
            Ok(serde_json::from_str(text.as_str()).unwrap())
        } else {
            Ok(response.json().await?)
        }
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
    use std::str::FromStr;

    use base64::engine::general_purpose::STANDARD;
    use base64_serde::base64_serde_type;
    use bigdecimal::BigDecimal;
    use time::OffsetDateTime;

    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use token_source::TokenSourceProvider;

    use crate::http::bigquery_client::{BigqueryClient, SCOPES};
    use crate::http::query;
    use crate::http::query::value::Decodable as QueryDecodable;
    use crate::http::table::{TableFieldMode, TableFieldSchema, TableFieldType, TableSchema};
    use crate::http::tabledata::list::Tuple;
    use crate::storage;
    use crate::storage::array::ArrayRef;
    use crate::storage::value::Decodable as StorageDecodable;

    base64_serde_type!(Base64Standard, STANDARD);

    #[ctor::ctor]
    fn init() {
        let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive("google_cloud_bigquery=trace".parse().unwrap());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    pub fn dataset_name(name: &str) -> String {
        format!("gcrbq_{}", name)
    }

    pub fn bucket_name(project: &str, name: &str) -> String {
        format!("{}_gcrbq_{}", project, name)
    }

    pub async fn create_client() -> (BigqueryClient, String) {
        let tsp = DefaultTokenSourceProvider::new(Config::default().with_scopes(&SCOPES))
            .await
            .unwrap();
        let cred = tsp.source_credentials.clone();
        let ts = tsp.token_source();
        let client = BigqueryClient::new(
            ts,
            "https://bigquery.googleapis.com",
            reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build(),
            false,
        );
        (client, cred.unwrap().project_id.unwrap())
    }

    #[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, PartialEq)]
    pub struct TestDataStruct {
        pub f1: bool,
        pub f2: Vec<i64>,
    }

    impl query::value::StructDecodable for TestDataStruct {
        fn decode(value: Tuple) -> Result<Self, query::value::Error> {
            let col = &value.f;
            Ok(Self {
                f1: bool::decode(&col[0].v)?,
                f2: Vec::<i64>::decode(&col[1].v)?,
            })
        }
    }

    impl storage::value::StructDecodable for TestDataStruct {
        fn decode_arrow(col: &[ArrayRef], row_no: usize) -> Result<TestDataStruct, storage::value::Error> {
            let f1 = bool::decode_arrow(&col[0], row_no)?;
            let f2 = Vec::<i64>::decode_arrow(&col[1], row_no)?;
            Ok(TestDataStruct { f1, f2 })
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug, PartialEq)]
    pub struct TestData {
        pub col_string: Option<String>,
        pub col_number: Option<BigDecimal>,
        pub col_number_array: Vec<BigDecimal>,
        #[serde(default, with = "time::serde::rfc3339::option")]
        pub col_timestamp: Option<OffsetDateTime>,
        pub col_json: Option<String>,
        pub col_json_array: Vec<String>,
        pub col_struct: Option<TestDataStruct>,
        pub col_struct_array: Vec<TestDataStruct>,
        #[serde(default, with = "Base64Standard")]
        pub col_binary: Vec<u8>,
    }

    impl query::value::StructDecodable for TestData {
        fn decode(value: Tuple) -> Result<Self, query::value::Error> {
            let col = &value.f;
            Ok(TestData {
                col_string: Option::<String>::decode(&col[0].v)?,
                col_number: Option::<BigDecimal>::decode(&col[1].v)?,
                col_number_array: Vec::<BigDecimal>::decode(&col[2].v)?,
                col_timestamp: Option::<OffsetDateTime>::decode(&col[3].v)?,
                col_json: Option::<String>::decode(&col[4].v)?,
                col_json_array: Vec::<String>::decode(&col[5].v)?,
                col_struct: Option::<TestDataStruct>::decode(&col[6].v)?,
                col_struct_array: Vec::<TestDataStruct>::decode(&col[7].v)?,
                col_binary: Vec::<u8>::decode(&col[8].v)?,
            })
        }
    }

    impl storage::value::StructDecodable for TestData {
        fn decode_arrow(col: &[ArrayRef], row_no: usize) -> Result<TestData, storage::value::Error> {
            Ok(TestData {
                col_string: Option::<String>::decode_arrow(&col[0], row_no)?,
                col_number: Option::<BigDecimal>::decode_arrow(&col[1], row_no)?,
                col_number_array: Vec::<BigDecimal>::decode_arrow(&col[2], row_no)?,
                col_timestamp: Option::<OffsetDateTime>::decode_arrow(&col[3], row_no)?,
                col_json: Option::<String>::decode_arrow(&col[4], row_no)?,
                col_json_array: Vec::<String>::decode_arrow(&col[5], row_no)?,
                col_struct: Option::<TestDataStruct>::decode_arrow(&col[6], row_no)?,
                col_struct_array: Vec::<TestDataStruct>::decode_arrow(&col[7], row_no)?,
                col_binary: Vec::<u8>::decode_arrow(&col[8], row_no)?,
            })
        }
    }

    impl TestData {
        pub fn default(index: usize, now: OffsetDateTime) -> TestData {
            TestData {
                col_string: Some(format!("test_{}", index)),
                col_number: Some(BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap()),
                col_number_array: vec![
                    BigDecimal::from_str(
                        "578960446186580977117854925043439539266.34992332820282019728792003956564819967",
                    )
                    .unwrap(),
                    BigDecimal::from_str(
                        "-578960446186580977117854925043439539266.34992332820282019728792003956564819968",
                    )
                    .unwrap(),
                ],
                col_timestamp: Some(now),
                col_json: Some("{\"field\":100}".to_string()),
                col_json_array: vec!["{\"field\":100}".to_string(), "{\"field\":200}".to_string()],
                col_struct: Some(TestDataStruct {
                    f1: true,
                    f2: vec![index as i64, 3, 4],
                }),
                col_struct_array: vec![
                    TestDataStruct {
                        f1: true,
                        f2: vec![index as i64, 5, 6],
                    },
                    TestDataStruct {
                        f1: false,
                        f2: vec![index as i64, 30, 40],
                    },
                ],
                col_binary: b"test".to_vec(),
            }
        }
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
                    data_type: TableFieldType::Bignumeric,
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
                TableFieldSchema {
                    name: "col_binary".to_string(),
                    data_type: TableFieldType::Bytes,
                    mode: Some(TableFieldMode::Required),
                    ..Default::default()
                },
            ],
        }
    }
}
