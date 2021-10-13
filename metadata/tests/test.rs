use metadata::on_gce;

#[tokio::test]
async fn test_on_gce() {
    let result = on_gce().await;
    assert_eq!(false, result);
    println!("executed first");
    let result = on_gce().await;
    assert_eq!(false, result);
    println!("executed second");
}
