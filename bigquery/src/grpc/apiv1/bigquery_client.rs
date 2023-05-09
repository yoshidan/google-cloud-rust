use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Response, Status, Streaming};
use google_cloud_gax::retry::{invoke_fn, RetrySetting};
use google_cloud_googleapis::bigquery::storage::v1::big_query_read_client::BigQueryReadClient;
use google_cloud_googleapis::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::bigquery::storage::v1::{
    AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse,
    CreateReadSessionRequest, CreateWriteStreamRequest, FinalizeWriteStreamRequest, FinalizeWriteStreamResponse,
    FlushRowsRequest, FlushRowsResponse, GetWriteStreamRequest, ReadRowsRequest, ReadRowsResponse, ReadSession,
    SplitReadStreamRequest, SplitReadStreamResponse, WriteStream,
};

#[derive(Clone)]
pub struct ReadClient {
    inner: BigQueryReadClient<Channel>,
}

impl ReadClient {
    pub fn new(inner: BigQueryReadClient<Channel>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_read_session(
        &mut self,
        req: CreateReadSessionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ReadSession>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let table = &req
            .read_session
            .ok_or(Status::invalid_argument("read_session is required"))?
            .table;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("read_session.table={table}"), req.clone());
                self.inner.create_read_session(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn read_rows(
        &mut self,
        req: ReadRowsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<ReadRowsResponse>>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let stream = &req.read_stream;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("read_stream={stream}"), req.clone());
                self.inner.read_rows(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn split_read_stream(
        &mut self,
        req: SplitReadStreamRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<SplitReadStreamResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let name = &req.name;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={name}"), req.clone());
                self.inner.split_read_stream(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }
}

#[derive(Clone)]
pub struct WriteClient {
    inner: BigQueryWriteClient<Channel>,
}

impl WriteClient {
    pub fn new(inner: BigQueryWriteClient<Channel>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn append_rows(
        &mut self,
        req: AppendRowsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<AppendRowsResponse>>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request("".to_string(), req.clone());
                self.inner.append_rows(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_write_stream(
        &mut self,
        req: CreateWriteStreamRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<WriteStream>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let parent = &req.parent;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={parent}"), req.clone());
                self.inner.create_write_stream(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_write_stream(
        &mut self,
        req: GetWriteStreamRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<WriteStream>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let name = &req.name;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={name}"), req.clone());
                self.inner.get_write_stream(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn finalize_write_stream(
        &mut self,
        req: FinalizeWriteStreamRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<FinalizeWriteStreamResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let name = &req.name;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={name}"), req.clone());
                self.inner.finalize_write_stream(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn batch_commit_write_streams(
        &mut self,
        req: BatchCommitWriteStreamsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<BatchCommitWriteStreamsResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let parent = &req.parent;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={parent}"), req.clone());
                self.inner
                    .batch_commit_write_streams(request)
                    .await
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn flush_rows(
        &mut self,
        req: FlushRowsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<FlushRowsResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let write_stream = &req.write_stream;
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("write_stream={write_stream}"), req.clone());
                self.inner.flush_rows(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }
}
