use std::sync::Arc;

use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::routine;
use crate::http::routine::list::{ListRoutinesRequest, ListRoutinesResponse, RoutineOverview};
use crate::http::routine::Routine;

#[derive(Clone)]
pub struct BigqueryRoutineClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryRoutineClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create(&self, metadata: &Routine) -> Result<Routine, Error> {
        let builder = routine::insert::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update(&self, metadata: &Routine) -> Result<Routine, Error> {
        let builder = routine::update::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, dataset_id: &str, routine_id: &str) -> Result<(), Error> {
        let builder =
            routine::delete::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, routine_id);
        self.inner.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(&self, project_id: &str, dataset_id: &str, routine_id: &str) -> Result<Routine, Error> {
        let builder = routine::get::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, routine_id);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        dataset_id: &str,
        req: &ListRoutinesRequest,
    ) -> Result<Vec<RoutineOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut routines = vec![];
        loop {
            let builder = routine::list::build(
                self.inner.endpoint(),
                self.inner.http(),
                project_id,
                dataset_id,
                req,
                page_token,
            );
            let response: ListRoutinesResponse = self.inner.send(builder).await?;
            routines.extend(response.routines);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(routines)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use serial_test::serial;
    use time::OffsetDateTime;

    use crate::http::bigquery_client::test::create_client;
    use crate::http::bigquery_routine_client::BigqueryRoutineClient;
    use crate::http::routine::list::ListRoutinesRequest;
    use crate::http::routine::{Argument, ArgumentKind, Language, Routine, RoutineReference, RoutineType};
    use crate::http::types::{StandardSqlDataType, TypeKind};

    #[tokio::test]
    #[serial]
    pub async fn crud_routine() {
        let (client, project) = create_client().await;
        let client = BigqueryRoutineClient::new(Arc::new(client));
        let _f1 = client
            .create(&Routine {
                etag: "".to_string(),
                routine_reference: RoutineReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_routine".to_string(),
                    routine_id: format!("AddFourAndDivide{}", OffsetDateTime::now_utc().unix_timestamp()),
                },
                routine_type: RoutineType::ScalarFunction,
                language: Some(Language::Sql),
                definition_body: "(x + 4) / y".to_string(),
                return_type: Some(StandardSqlDataType {
                    type_kind: TypeKind::Float64,
                }),
                arguments: Some(vec![
                    Argument {
                        name: Some("x".to_string()),
                        argument_kind: Some(ArgumentKind::FixedType),
                        mode: None,
                        data_type: StandardSqlDataType {
                            type_kind: TypeKind::Int64,
                        },
                    },
                    Argument {
                        name: Some("y".to_string()),
                        argument_kind: Some(ArgumentKind::FixedType),
                        mode: None,
                        data_type: StandardSqlDataType {
                            type_kind: TypeKind::Int64,
                        },
                    },
                ]),
                ..Default::default()
            })
            .await
            .unwrap();

        let _f2 = client
            .create(&Routine {
                etag: "".to_string(),
                routine_reference: RoutineReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_routine".to_string(),
                    routine_id: format!("ExternalTable{}", OffsetDateTime::now_utc().unix_timestamp()),
                },
                routine_type: RoutineType::TableValuedFunction,
                language: Some(Language::Sql),
                definition_body: format!(
                    "SELECT * FROM `{}.rust_test_external_table.csv_table` WHERE string_field_0 = x",
                    project
                ),
                arguments: Some(vec![Argument {
                    name: Some("x".to_string()),
                    argument_kind: Some(ArgumentKind::FixedType),
                    mode: None,
                    data_type: StandardSqlDataType {
                        type_kind: TypeKind::String,
                    },
                }]),
                ..Default::default()
            })
            .await
            .unwrap();

        let _f3 = client
            .create(&Routine {
                etag: "".to_string(),
                routine_reference: RoutineReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_routine".to_string(),
                    routine_id: format!("Procedure{}", OffsetDateTime::now_utc().unix_timestamp()),
                },
                routine_type: RoutineType::Procedure,
                definition_body: format!(
                    "
            DECLARE id STRING;
            SET id = GENERATE_UUID();
            INSERT INTO `{}.rust_test_external_table.csv_table` VALUES(id, name)
            ",
                    project
                ),
                arguments: Some(vec![Argument {
                    name: Some("name".to_string()),
                    argument_kind: Some(ArgumentKind::FixedType),
                    mode: None,
                    data_type: StandardSqlDataType {
                        type_kind: TypeKind::String,
                    },
                }]),
                ..Default::default()
            })
            .await
            .unwrap();

        let all = client
            .list(project.as_str(), "rust_test_routine", &ListRoutinesRequest::default())
            .await
            .unwrap();
        for f in all {
            let f = f.routine_reference;
            let f = client
                .get(f.project_id.as_str(), f.dataset_id.as_str(), f.routine_id.as_str())
                .await
                .unwrap()
                .routine_reference;
            client
                .delete(f.project_id.as_str(), f.dataset_id.as_str(), f.routine_id.as_str())
                .await
                .unwrap();
        }
    }
}
