use crate::apiv1::default_setting;
use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::pubsub::v1::schema_service_client::SchemaServiceClient;
use google_cloud_googleapis::pubsub::v1::{
    CreateSchemaRequest, GetSchemaRequest, ListSchemasRequest, Schema, ValidateMessageRequest,
    ValidateMessageResponse, ValidateSchemaRequest, ValidateSchemaResponse,
};
use google_cloud_googleapis::{Code, Status};
use google_cloud_grpc::conn::Channel;
use tonic::Response;

#[derive(Clone)]
pub struct SchemaClient {
    inner: SchemaServiceClient<Channel>,
}

impl SchemaClient {
    /// create new publisher client
    pub fn new(inner: SchemaServiceClient<Channel>) -> SchemaClient {
        SchemaClient { inner }
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// create_schema creates a schema.
    pub async fn create_schema(
        &mut self,
        req: CreateSchemaRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Schema>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("parent={}", parent), req.clone());
                client
                    .create_schema(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_schema gets a schema.
    pub async fn get_schema(
        &mut self,
        req: GetSchemaRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Schema>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .get_schema(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_schemas lists matching topics.
    pub async fn list_schemas(
        &mut self,
        mut req: ListSchemasRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Schema>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("project={}", project), req.clone());
                    client
                        .list_schemas(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            all.extend(response.topics.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// delete_schema deletes a schema.
    pub async fn delete_schema(
        &mut self,
        req: GetSchemaRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .delete_schema(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// validate_schema deletes a schema.
    pub async fn validate_schema(
        &mut self,
        req: ValidateSchemaRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ValidateSchemaResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("parent={}", parent), req.clone());
                client
                    .validate_schema(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// validate_message validates a message against a schema.
    pub async fn validate_message(
        &mut self,
        req: ValidateMessageRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ValidateMessageResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("parent={}", parent), req.clone());
                client
                    .validate_message(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}
