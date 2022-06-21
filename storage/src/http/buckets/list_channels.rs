use crate::http::channels::Channel;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

/// Request message for ListChannels.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
}

/// The result of a call to Channels.ListChannels
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsResponse {
    /// The list of notification channels for a bucket.
    pub items: Vec<Channel>,
}

#[allow(dead_code)]
pub(crate) fn build(client: &Client, req: &ListChannelsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/channels", BASE_URL, req.bucket.escape());
    client.get(url)
}
