use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Escape};
use crate::http::channels::Channel;

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

pub(crate) fn build(client: &Client, req: &ListChannelsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/channels", BASE_URL, req.bucket.escape());
    client.get(url)
}