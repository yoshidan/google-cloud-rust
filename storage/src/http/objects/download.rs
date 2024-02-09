use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::objects::get::GetObjectRequest;
use crate::http::Escape;

#[derive(Default)]
pub struct Range(pub Option<u64>, pub Option<u64>);

impl Range {
    /// Range: bytes=0-1999 (first 2000 bytes)
    /// Range: bytes=-2000 (last 2000 bytes)
    /// Range: bytes=2000- (from byte 2000 to end of file)
    fn with_header(&self, builder: RequestBuilder) -> RequestBuilder {
        if let Some(from) = self.0 {
            if let Some(to) = self.1 {
                builder.header("Range", format!("bytes={from}-{to}"))
            } else {
                builder.header("Range", format!("bytes={from}-"))
            }
        } else if let Some(reverse_from) = self.1 {
            builder.header("Range", format!("bytes=-{reverse_from}"))
        } else {
            builder
        }
    }
}

pub(crate) fn build(base_url: &str, client: &Client, req: &GetObjectRequest, range: &Range) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}?alt=media", base_url, req.bucket.escape(), req.object.escape());
    let builder = range.with_header(client.get(url).query(&req));
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
