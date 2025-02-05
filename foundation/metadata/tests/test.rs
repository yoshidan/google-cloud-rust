use gcloud_metadata::{email, on_gce, Error};

#[tokio::test]
async fn test_on_gce() {
    let result = on_gce().await;
    assert!(!result);
    println!("executed first");
    let result = on_gce().await;
    assert!(!result);
    println!("executed second");
}

#[tokio::test]
async fn test_email() {
    let result = email("default").await;
    if let Err(e) = result {
        match e {
            Error::HttpError(e) => println!("http error {e:?}"),
            _ => unreachable!(),
        }
    } else {
        unreachable!()
    }
}
