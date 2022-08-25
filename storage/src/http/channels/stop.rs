use crate::http::channels::WatchableChannel;
use reqwest::{Client, RequestBuilder};

/// Request message for StopChannel.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopChannelRequest {
    /// The channel to be stopped.
    pub channel: WatchableChannel,
}

#[allow(dead_code)]
pub(crate) fn build(base_url: &str, client: &Client, req: &StopChannelRequest) -> RequestBuilder {
    let url = format!("{}/channels/stop", base_url);
    client.post(url).json(&req.channel)
}
