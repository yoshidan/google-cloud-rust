use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::channels::Channel;
use crate::http::Escape;

/// Request message for ListChannels.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
}

/// The result of a call to Channels.ListChannels
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsResponse {
    /// The list of notification channels for a bucket.
    pub items: Vec<Channel>,
}

#[allow(dead_code)]
pub(crate) fn build(base_url: &str, client: &Client, req: &ListChannelsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/channels", base_url, req.bucket.escape());
    client.get(url)
}
