use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{invoke_fn, RetrySetting};
use google_cloud_googleapis::devtools::artifact_registry::v1::artifact_registry_client::ArtifactRegistryClient;
use google_cloud_googleapis::devtools::artifact_registry::v1::{
    CreateRepositoryRequest, CreateTagRequest, DeletePackageRequest, DeleteRepositoryRequest, DeleteTagRequest,
    DeleteVersionRequest, DockerImage, File, GetDockerImageRequest, GetFileRequest, GetMavenArtifactRequest,
    GetNpmPackageRequest, GetPackageRequest, GetProjectSettingsRequest, GetPythonPackageRequest, GetRepositoryRequest,
    GetTagRequest, GetVersionRequest, ImportAptArtifactsRequest, ImportAptArtifactsResponse, ImportYumArtifactsRequest,
    ListDockerImagesRequest, ListDockerImagesResponse, ListFilesRequest, ListFilesResponse, ListMavenArtifactsRequest,
    ListMavenArtifactsResponse, ListNpmPackagesRequest, ListNpmPackagesResponse, ListPackagesRequest,
    ListPackagesResponse, ListPythonPackagesRequest, ListPythonPackagesResponse, ListRepositoriesRequest,
    ListRepositoriesResponse, ListTagsRequest, ListTagsResponse, ListVersionsRequest, ListVersionsResponse,
    MavenArtifact, NpmPackage, Package, ProjectSettings, PythonPackage, Repository, Tag, UpdateProjectSettingsRequest,
    UpdateRepositoryRequest, UpdateTagRequest, Version, YumArtifact,
};
use google_cloud_googleapis::iam::v1::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest};
use google_cloud_longrunning::autogen::operations_client::OperationsClient;
use google_cloud_longrunning::longrunning::Operation;
use std::time::Duration;

fn default_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(60)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown],
    }
}

#[derive(Clone)]
pub struct Client {
    inner: ArtifactRegistryClient<Channel>,
    lro_client: OperationsClient,
}

impl Client {
    pub fn new(inner: ArtifactRegistryClient<Channel>, lro_client: OperationsClient) -> Self {
        Self {
            inner: inner.max_decoding_message_size(i32::MAX as usize),
            lro_client,
        }
    }

    /// Get project settings
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#getprojectsettingsrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects/getProjectSettings
    ///
    /// Note: This v1 endpoint doesn't seem to be working. V1 beta to be used or wait for next version.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_project_settings(
        &mut self,
        req: GetProjectSettingsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ProjectSettings, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_project_settings(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Update project settings
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#updateprojectsettingsrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects/updateProjectSettings
    ///
    /// Note: This v1 endpoint doesn't seem to be working. V1 beta to be used or wait for next version.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update_project_settings(
        &mut self,
        req: UpdateProjectSettingsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ProjectSettings, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let project_settings_name = match req.project_settings {
            None => "".to_string(),
            Some(ref s) => s.name.to_string(),
        };

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("project_settings.name={}", project_settings_name), req.clone());
                client
                    .update_project_settings(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Create repository
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#createrepositoryrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/create
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_repository(
        &mut self,
        req: CreateRepositoryRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Operation<Repository>, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client.create_repository(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()))
    }

    /// Get repository
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.GetRepositoryRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_repository(
        &mut self,
        req: GetRepositoryRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Repository, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("repository.name={}", req.name), req.clone());
                client
                    .get_repository(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// List repositories
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ListRepositoriesRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_repositories(
        &mut self,
        req: ListRepositoriesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListRepositoriesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_repositories(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Update repository
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.UpdateRepositoryRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/patch
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update_repository(
        &mut self,
        req: UpdateRepositoryRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Repository, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let repository_name = match req.repository {
            None => "".to_string(),
            Some(ref r) => r.name.to_string(),
        };

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("repository.name={}", repository_name), req.clone());
                client
                    .update_repository(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Delete repository
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.DeleteRepositoryRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/delete
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_repository(
        &mut self,
        req: DeleteRepositoryRequest,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .delete_repository(request)
                    .await
                    .map(|_r| ())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get IAM policy
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ArtifactRegistry.GetIamPolicy
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/getIamPolicy
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_iam_policy(
        &mut self,
        req: GetIamPolicyRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Policy, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("resource={}", req.resource), req.clone());
                client
                    .get_iam_policy(request)
                    .await
                    .map(|p| p.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Set IAM policy
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ArtifactRegistry.SetIamPolicy
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/setIamPolicy
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn set_iam_policy(
        &mut self,
        req: SetIamPolicyRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Policy, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("resource={}", req.resource), req.clone());
                client
                    .set_iam_policy(request)
                    .await
                    .map(|p| p.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get locations
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.iam.v1#google.iam.v1.TestIamPermissionsRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories/testIamPermissions
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_iam_permissions(
        &mut self,
        req: TestIamPermissionsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("resource={}", req.resource), req.clone());
                client
                    .test_iam_permissions(request)
                    .await
                    .map(|s| s.into_inner().permissions)
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// List Docker images
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listdockerimagesrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.dockerImages/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_docker_images(
        &mut self,
        req: ListDockerImagesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListDockerImagesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_docker_images(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get Docker image
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listdockerimagesrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.dockerImages/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_docker_image(
        &mut self,
        req: GetDockerImageRequest,
        retry: Option<RetrySetting>,
    ) -> Result<DockerImage, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_docker_image(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Import APT artifacts
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ImportAptArtifactsRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.aptArtifacts/import
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn import_apt_artifacts(
        &mut self,
        req: ImportAptArtifactsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Operation<ImportAptArtifactsResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client.import_apt_artifacts(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()))
    }

    /// File get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.GetFileRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.files/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_file(&mut self, req: GetFileRequest, retry: Option<RetrySetting>) -> Result<File, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_file(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Files list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listfilesrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.files/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_files(
        &mut self,
        req: ListFilesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListFilesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_files(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Maven artifact get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#getmavenartifactrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.mavenArtifacts/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_maven_artifact(
        &mut self,
        req: GetMavenArtifactRequest,
        retry: Option<RetrySetting>,
    ) -> Result<MavenArtifact, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_maven_artifact(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Maven artifacts list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listmavenartifactsrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.mavenArtifacts/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_maven_artifacts(
        &mut self,
        req: ListMavenArtifactsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListMavenArtifactsResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_maven_artifacts(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// NPM package get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#getnpmpackagerequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.npmPackages/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_npm_package(
        &mut self,
        req: GetNpmPackageRequest,
        retry: Option<RetrySetting>,
    ) -> Result<NpmPackage, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_npm_package(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// NPM packages list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listnpmpackagesrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.npmPackages/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_npm_packages(
        &mut self,
        req: ListNpmPackagesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListNpmPackagesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_npm_packages(request)
                    .await
                    .map(|d| d.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package delete
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.DeletePackageRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages/delete
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_package(
        &mut self,
        req: DeletePackageRequest,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .delete_package(request)
                    .await
                    .map(|_o| ())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.GetPackageRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_package(
        &mut self,
        req: GetPackageRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Package, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_package(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Packages list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ListPackagesRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_packages(
        &mut self,
        req: ListPackagesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListPackagesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_packages(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Tag create
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#createtagrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.tags/create
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_package_tag(
        &mut self,
        req: CreateTagRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Tag, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .create_tag(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Tag get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#gettagrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.tags/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_package_tag(&mut self, req: GetTagRequest, retry: Option<RetrySetting>) -> Result<Tag, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_tag(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Tag delete
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#deletetagrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.tags/delete
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_package_tag(
        &mut self,
        req: DeleteTagRequest,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .delete_tag(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Tag list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listtagsrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.tags/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_package_tags(
        &mut self,
        req: ListTagsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListTagsResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_tags(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Tag update
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#updatetagrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.tags/patch
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update_package_tag(
        &mut self,
        req: UpdateTagRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Tag, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let tag_name = match req.tag {
            None => "".to_string(),
            Some(ref t) => t.name.to_string(),
        };

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("tag.name={}", tag_name), req.clone());
                client
                    .update_tag(request)
                    .await
                    .map(|o| o.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Version delete
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.DeleteVersionRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.versions/delete
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_package_version(
        &mut self,
        req: DeleteVersionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .delete_version(request)
                    .await
                    .map(|_o| ())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Version get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.GetVersionRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.versions/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_package_version(
        &mut self,
        req: GetVersionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Version, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_version(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Package Version list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#google.devtools.artifactregistry.v1.ListVersionsRequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.packages.versions/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_package_versions(
        &mut self,
        req: ListVersionsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListVersionsResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_versions(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Python packages list
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#listpythonpackagesrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.pythonPackages/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_python_packages(
        &mut self,
        req: ListPythonPackagesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListPythonPackagesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_python_packages(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Python package get
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#getpythonpackagerequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.pythonPackages/get
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_python_package(
        &mut self,
        req: GetPythonPackageRequest,
        retry: Option<RetrySetting>,
    ) -> Result<PythonPackage, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_python_package(request)
                    .await
                    .map(|r| r.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Yum Artifacts import
    ///
    /// https://cloud.google.com/artifact-registry/docs/reference/rpc/google.devtools.artifactregistry.v1#importyumartifactsrequest
    /// REST reference: https://cloud.google.com/artifact-registry/docs/reference/rest/v1/projects.locations.repositories.yumArtifacts/import
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn import_yum_artifacts(
        &mut self,
        req: ImportYumArtifactsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Operation<YumArtifact>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client.import_yum_artifacts(request).await.map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()))
    }
}
