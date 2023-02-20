use crate::http::objects::download::Range;
use crate::http::Error;
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE};
use reqwest::Client;

#[derive(thiserror::Error, Debug)]
pub enum ChunkError {
    InvalidRange(usize, usize),
    ZeroTotalObjectSize,
}

pub enum UploadStatus {
    Ok,
    ResumeIncomplete,
}

pub enum TotalSize {
    Unknown,
    Known(usize),
}

impl ToString for TotalSize {
    fn to_string(&self) -> String {
        match self {
            TotalSize::Unknown => "*".to_string(),
            TotalSize::Known(size) => size.to_string(),
        }
    }
}

pub struct ChunkSize {
    first_byte: usize,
    last_byte: usize,
    total_object_size: TotalSize,
}

impl ToString for ChunkSize {
    fn to_string(&self) -> String {
        format!(
            "bytes {}-{}/{}",
            self.first_byte,
            self.last_byte,
            self.total_object_size.to_string()
        )
    }
}

impl ChunkSize {
    pub fn new(first_byte: usize, last_byte: usize, total_object_size: TotalSize) -> Result<Self, ChunkError> {
        if let TotalSize(size) = total_object_size {
            if size == 0 {
                return Err(ChunkError::ZeroTotalObjectSize);
            }
        }
        if first_byte >= last_byte {
            return Err(ChunkError::InvalidRange(first_byte, last_byte));
        }
        let size = last_byte - first_byte + 1;
        if size % (256 * 1024) != 0 {
            tracing::warn!("The chunk size should be multiple of 256KiB. size = {}", size);
        }

        Ok(Self {
            first_byte,
            last_byte,
            total_object_size,
        })
    }

    pub fn size(&self) -> usize {
        self.last_byte - self.first_byte + 1
    }
}

#[derive(Clone)]
pub struct ResumableUploadClient {
    session_url: String,
    http: Client,
}

impl ResumableUploadClient {
    pub fn new(session_url: String, http: Client) -> Self {
        Self { session_url, http }
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#single-chunk-upload
    pub fn upload_single_chunk<T>(&self, data: T, size: usize) -> Result<(), Error> {
        let response = self
            .http
            .put(&self.session_url)
            .header(CONTENT_LENGTH, size)
            .body(data)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from_response(response).await)
        }
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#chunked-upload
    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#resume-upload
    pub fn upload_multiple_chunk<T>(&self, data: T, size: &ChunkSize) -> Result<UploadStatus, Error> {
        let response = self
            .http
            .put(&self.session_url)
            .header(CONTENT_RANGE, size.to_string())
            .header(CONTENT_LENGTH, size.size())
            .body(data)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(UploadStatus::Ok)
        } else if response.status() == 308 {
            Ok(UploadStatus::ResumeIncomplete)
        } else {
            Err(Error::from_response(response).await)
        }
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#status-check
    pub fn status(&self, object_size: &TotalSize) -> Result<UploadStatus, Error> {
        let response = self
            .http
            .put(&self.session_url)
            .header(CONTENT_RANGE, format! {"bytes */{}", object_size.to_string()})
            .send()
            .await?;
        if response.status().is_success() {
            Ok(UploadStatus::Ok)
        } else if response.status() == 308 {
            Ok(UploadStatus::ResumeIncomplete)
        } else {
            Err(Error::from_response(response).await)
        }
    }

    /// https://cloud.google.com/storage/docs/performing-resumable-uploads#cancel-upload
    pub fn cancel(&self) -> Result<(), Error> {
        let response = self.http.delete(&self.session_url).send().await?;
        if response.status() == 499 {
            Ok(())
        } else {
            Err(Error::from_response(response).await)
        }
    }
}
