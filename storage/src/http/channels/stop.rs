use crate::http::channels::WatchableChannel;
use crate::http::BASE_URL;
use reqwest::{Client, RequestBuilder};

/// Request message for StopChannel.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopChannelRequest {
    /// The channel to be stopped.
    pub channel: WatchableChannel,
}

pub(crate) fn build(client: &Client, req: &StopChannelRequest) -> RequestBuilder {
    let url = format!("{}/channels/stop", BASE_URL);
    client.post(url).json(&req.channel)
}
