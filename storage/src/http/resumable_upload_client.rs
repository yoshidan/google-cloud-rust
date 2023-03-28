use std::fmt;

use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE};
use reqwest::{Body, Client, Response};

use crate::http::{check_response_status, objects::Object, Error};

#[derive(thiserror::Error, Debug)]
pub enum ChunkError {
    #[error("invalid range: first={0} last={1}")]
    InvalidRange(u64, u64),
    #[error("total object size must not be zero")]
    ZeroTotalObjectSize,
    #[error("last byte must be less than total object size: last={0} total={1}")]
    InvalidLastBytes(u64, u64),
}

#[derive(PartialEq, Debug)]
pub enum UploadStatus {
    Ok(Object),
    ResumeIncomplete,
}

#[derive(Clone, Debug)]
pub struct ChunkSize {
    first_byte: u64,
    last_byte: u64,
    total_object_size: Option<u64>,
}

impl fmt::Display for ChunkSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.first_byte == self.last_byte {
            write!(f, "bytes */")?;
        } else {
            write!(f, "bytes {}-{}/", self.first_byte, self.last_byte)?;
        }

        match self.total_object_size {
            Some(total_object_size) => write!(f, "{total_object_size}"),
            None => write!(f, "*"),
        }
    }
}

impl ChunkSize {
    pub fn new(first_byte: u64, last_byte: u64, total_object_size: Option<u64>) -> ChunkSize {
        Self {
            first_byte,
            last_byte,
            total_object_size,
        }
    }

    pub fn size(&self) -> u64 {
        if self.first_byte == self.last_byte {
            0
        } else {
            self.last_byte - self.first_byte + 1
        }
    }
}

#[derive(Clone)]
pub struct ResumableUploadClient {
    session_url: String,
    http: Client,
}

impl ResumableUploadClient {
    pub fn url(&self) -> &str {
        self.session_url.as_str()
    }

    pub fn new(session_url: String, http: Client) -> Self {
        Self { session_url, http }
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#single-chunk-upload
    pub async fn upload_single_chunk<T: Into<Body>>(&self, data: T, size: usize) -> Result<(), Error> {
        let response = self
            .http
            .put(&self.session_url)
            .header(CONTENT_LENGTH, size)
            .body(data)
            .send()
            .await?;
        check_response_status(response).await?;
        Ok(())
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#chunked-upload
    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#resume-upload
    pub async fn upload_multiple_chunk<T: Into<Body>>(&self, data: T, size: &ChunkSize) -> Result<UploadStatus, Error> {
        let response = self
            .http
            .put(&self.session_url)
            .header(CONTENT_RANGE, size.to_string())
            .header(CONTENT_LENGTH, size.size())
            .body(data)
            .send()
            .await?;
        Self::map_resume_response(response).await
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#status-check
    pub async fn status(&self, object_size: Option<u64>) -> Result<UploadStatus, Error> {
        self.upload_multiple_chunk("", &ChunkSize::new(0, 0, object_size)).await
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#cancel-upload
    pub async fn cancel(self) -> Result<(), Error> {
        let response = self
            .http
            .delete(&self.session_url)
            .header(CONTENT_LENGTH, 0)
            .send()
            .await?;
        if response.status() == 499 {
            Ok(())
        } else {
            check_response_status(response).await?;
            Ok(())
        }
    }

    async fn map_resume_response(response: Response) -> Result<UploadStatus, Error> {
        if response.status() == 308 {
            Ok(UploadStatus::ResumeIncomplete)
        } else {
            let response = check_response_status(response).await?;
            Ok(UploadStatus::Ok(response.json::<Object>().await?))
        }
    }
}
