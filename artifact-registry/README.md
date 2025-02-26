# google-cloud-artifact-registry

Google Cloud Platform Artifact Registry Client library.

[![crates.io](https://img.shields.io/crates/v/gcloud-artifact-registry.svg)](https://crates.io/crates/google-cloud-artifact-registry)

* [About Artifact Registry](https://cloud.google.com/artifact-registry/)
* [JSON API Documentation](https://cloud.google.com/artifact-registry/docs/reference/rest)
* [RPC Documentation](https://cloud.google.com/artifact-registry/docs/reference/rpc)

## Installation

```toml
[dependencies]
google-cloud-artifact-registry = {package="gcloud-artifact-registry", version="1.0.0" }
```

## Quickstart

### Authentication
There are two ways to create a client that is authenticated against the google cloud.

#### Automatically

The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
from a metadata server.

 ```rust
 use google_cloud_artifact_registry::client::{Client, ClientConfig};

 async fn run() {
     let config = ClientConfig::default().with_auth().await.unwrap();
     let client = Client::new(config);
 }
 ```

 #### Manually

 When you can't use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
 you can parse your own version of the 'credentials-file' and use it like that:

 ```rust
 use google_cloud_auth::credentials::CredentialsFile;
 // or google_artifact_registry::client::google_cloud_auth::credentials::CredentialsFile
 use google_cloud_artifact_registry::client::{Client, ClientConfig};

 async fn run(cred: CredentialsFile) {
    let config = ClientConfig::default().with_credentials(cred).await.unwrap();
    let client = Client::new(config);
 }
 ```

 ### Usage

 #### Repository operations

 ```rust
 use std::collections::HashMap;
 use prost_types::FieldMask;
 use google_cloud_artifact_registry::client::{Client, ClientConfig};
 use google_cloud_googleapis::devtools::artifact_registry::v1::{CreateRepositoryRequest, DeleteRepositoryRequest, GetRepositoryRequest, ListRepositoriesRequest, Repository, UpdateRepositoryRequest};
 use google_cloud_googleapis::devtools::artifact_registry::v1::repository::Format;
 use google_cloud_googleapis::iam::v1::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest};

 async fn run(config: ClientConfig) {

     // Create client
     let mut client = Client::new(config).await.unwrap();

     // Repository
     // create
     match client
         .create_repository(
             CreateRepositoryRequest {
                 parent: "projects/qovery-gcp-tests/locations/europe-west9".to_string(),
                 repository_id: "repository-for-documentation".to_string(),
                 repository: Some(Repository {
                     name: "repository-for-documentation".to_string(),
                     format: Format::Docker.into(),
                     description: "Example repository for documentation".to_string(),
                     labels: HashMap::from_iter(vec![
                         ("a_label".to_string(), "a_label_value".to_string()),
                         ("another_label".to_string(), "another_label_value".to_string()),
                     ]),
                     ..Default::default()
                 }),
             },
             None,
         )
         .await
     {
         Ok(mut r) => println!("Created repository {:?}", r.wait(None).await.unwrap()),
         Err(err) => panic!("err: {:?}", err),
     };

     // update
     match client
        .update_repository(
            UpdateRepositoryRequest {
                 repository: Some(Repository {
                     name: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                         .to_string(),
                     description: "updated description".to_string(),
                     labels: HashMap::from_iter(vec![(
                         "yet_another_label".to_string(),
                         "yet_another_label_value".to_string(),
                     )]),
                     ..Default::default()
                 }),
                 update_mask: Some(FieldMask {
                     paths: vec!["description".to_string(), "labels".to_string()],
                 }),
             },
             None,
         )
         .await
     {
         Ok(r) => println!("Updated repository {:?}", r),
         Err(err) => panic!("err: {:?}", err),
     };

     // list
     match client
         .list_repositories(
             ListRepositoriesRequest {
                 parent: "projects/qovery-gcp-tests/locations/europe-west9".to_string(),
                 page_size: 100,
                 page_token: "".to_string(),
             },
             None,
         )
         .await
     {
         Ok(response) => {
             println!("List repositories");
             for r in response.repositories {
                 println!("- {:?}", r);
             }
         }
         Err(err) => panic!("err: {:?}", err),
     }

     // get
     match client
         .get_repository(
             GetRepositoryRequest {
                 name: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
            },
             None,
         )
         .await
     {
         Ok(r) => println!("Get repository {:?}", r),
         Err(err) => panic!("err: {:?}", err),
     }

     // delete
     match client
         .delete_repository(
             DeleteRepositoryRequest {
                 name: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
             },
             None,
         )
         .await
     {
         Ok(r) => println!("Delete repository `repository-for-documentation`"),
         Err(err) => panic!("err: {:?}", err),
     }

     // get repository IAM policy
     match client
         .get_iam_policy(
             GetIamPolicyRequest {
                 resource: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
                 ..Default::default()
             },
             None,
         )
         .await
     {
         Ok(policy) => println!("Get IAM Policy for `repository-for-documentation` {:?}", policy),
         Err(err) => panic!("err: {:?}", err),
     }

     // update repository IAM policy
     match client
         .set_iam_policy(
             SetIamPolicyRequest {
                 resource: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
                 policy: Some(Policy {
                     version: 3,
                     ..Default::default()
                 }),
                 update_mask: Some(FieldMask {
                     paths: vec!["policy.version".to_string()],
                 }),
             },
             None,
         )
         .await
     {
         Ok(policy) => println!("Update IAM Policy for `repository-for-documentation` {:?}", policy),
         Err(err) => panic!("err: {:?}", err),
     }

 // test IAM repository IAM policy
     match client
         .test_iam_permissions(
             TestIamPermissionsRequest {
                 resource: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
                 ..Default::default()
             },
             None,
         )
         .await
     {
         Ok(permissions) => {
             println!("Test permissions for `repository-for-documentation`, permissions:");
             for p in permissions {
                 println!("- Permission: {}", p);
             }
         }
         Err(err) => panic!("err: {:?}", err),
     }

 }
 ```

 #### Docker images operations

 ```rust
 use google_cloud_artifact_registry::client::{Client, ClientConfig};
 use google_cloud_googleapis::devtools::artifact_registry::v1::{GetDockerImageRequest, ListDockerImagesRequest};

 async fn run(config: ClientConfig)  {

     // Create client.
     let mut client = Client::new(config).await.unwrap();

     // Docker images
     // list
     match client
         .list_docker_images(
             ListDockerImagesRequest {
                 parent: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation"
                     .to_string(),
                 ..Default::default()
             },
             None,
         )
         .await
     {
         Ok(response) => {
             println!("Docker images for repository `repository-for-documentation`: ");
             for image in response.docker_images {
                 println!("- Image: {:?}", image);
             }
         }
         Err(e) => {
             println!("Error: {}", e);
             println!("Error details: {:?}", e.metadata())
         }
     }

     // get
     let result = client.get_docker_image(GetDockerImageRequest {
         name: "projects/qovery-gcp-tests/locations/europe-west9/repositories/repository-for-documentation/dockerImages/quickstart-image@sha256:2571d3a406da0ecafff96a9c707bc2eba954352dabc85dd918af2e3ec40c263a".to_string(),
     }, None).await;

     match result {
         Ok(d) => {
             println!("Image: {:?}", d);
         }
         Err(e) => {
             println!("Error: {}", e);
             println!("Error details: {:?}", e.metadata())
         }
     }
 }
 ```