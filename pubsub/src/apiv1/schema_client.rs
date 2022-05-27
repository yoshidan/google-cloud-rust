use crate::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::Response;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::{invoke, RetrySetting};
use google_cloud_googleapis::pubsub::v1::schema_service_client::SchemaServiceClient;
use google_cloud_googleapis::pubsub::v1::{
    CreateSchemaRequest, DeleteSchemaRequest, GetSchemaRequest, ListSchemasRequest, Schema, ValidateMessageRequest,
    ValidateMessageResponse, ValidateSchemaRequest, ValidateSchemaResponse,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct SchemaClient {
    cm: Arc<ConnectionManager>,
}

impl SchemaClient {
    /// create new publisher client
    pub fn new(cm: ConnectionManager) -> SchemaClient {
        SchemaClient { cm: Arc::new(cm) }
    }

    fn client(&self) -> SchemaServiceClient<Channel> {
        SchemaServiceClient::new(self.cm.conn())
    }

    /// create_schema creates a schema.
    pub async fn create_schema(
        &self,
        req: CreateSchemaRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Schema>, Status> {
        let parent = &req.parent;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("parent={}", parent), req.clone());
            client.create_schema(request).await.map_err(|e| e.into())
        };
        invoke(cancel, retry, action).await
    }

    /// get_schema gets a schema.
    pub async fn get_schema(
        &self,
        req: GetSchemaRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Schema>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.get_schema(request).await.map_err(|e| e.into())
        };
        invoke(cancel, retry, action).await
    }

    /// list_schemas lists matching topics.
    pub async fn list_schemas(
        &self,
        mut req: ListSchemasRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Schema>, Status> {
        let project = &req.parent;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("project={}", project), req.clone());
                client
                    .list_schemas(request)
                    .await
                    .map_err(|e| e.into())
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), retry.clone(), action).await?;
            all.extend(response.schemas.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// delete_schema deletes a schema.
    pub async fn delete_schema(
        &self,
        req: DeleteSchemaRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.delete_schema(request).await.map_err(|e| e.into())
        };
        invoke(cancel, retry, action).await
    }

    /// validate_schema deletes a schema.
    pub async fn validate_schema(
        &self,
        req: ValidateSchemaRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ValidateSchemaResponse>, Status> {
        let parent = &req.parent;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("parent={}", parent), req.clone());
            client.validate_schema(request).await.map_err(|e| e.into())
        };
        invoke(cancel, retry, action).await
    }

    /// validate_message validates a message against a schema.
    pub async fn validate_message(
        &self,
        req: ValidateMessageRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ValidateMessageResponse>, Status> {
        let parent = &req.parent;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("parent={}", parent), req.clone());
            client.validate_message(request).await.map_err(|e| e.into())
        };
        invoke(cancel, retry, action).await
    }
}
