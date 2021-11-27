use tonic::{IntoRequest, Request};

pub fn create_request<T>(
    param_string: String,
    token: &Option<String>,
    into_request: impl IntoRequest<T>,
) -> Request<T> {
    let mut request = into_request.into_request();
    let target = request.metadata_mut();
    target.append("x-goog-request-params", param_string.parse().unwrap());
    if token.is_some() {
        target.insert("authorization", token.as_ref().unwrap().parse().unwrap());
    };
    request
}
