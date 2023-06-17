use std::sync::Arc;

use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::row_access_policy;
use crate::http::row_access_policy::list::{
    ListRowAccessPoliciesRequest, ListRowAccessPoliciesResponse, RowAccessPolicyOverview,
};
use crate::http::table::get_iam_policy::GetIamPolicyRequest;
use crate::http::table::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::types::Policy;

#[derive(Debug, Clone)]
pub struct BigqueryRowAccessPolicyClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryRowAccessPolicyClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/rowAccessPolicies/getIamPolicy
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_iam_policy(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        policy_id: &str,
        req: &GetIamPolicyRequest,
    ) -> Result<Policy, Error> {
        let builder = row_access_policy::get_iam_policy::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            policy_id,
            req,
        );
        self.inner.send(builder).await
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/rowAccessPolicies/testIamPermissions
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_iam_permissions(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        policy_id: &str,
        req: &TestIamPermissionsRequest,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let builder = row_access_policy::test_iam_permissions::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            policy_id,
            req,
        );
        self.inner.send(builder).await
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/rowAccessPolicies/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &ListRowAccessPoliciesRequest,
    ) -> Result<Vec<RowAccessPolicyOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut policies = vec![];
        loop {
            let builder = row_access_policy::list::build(
                self.inner.endpoint(),
                self.inner.http(),
                project_id,
                dataset_id,
                table_id,
                req,
                page_token,
            );
            let response: ListRowAccessPoliciesResponse = self.inner.send(builder).await?;
            if let Some(data) = response.row_access_policies {
                policies.extend(data);
            }
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(policies)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use serial_test::serial;

    use crate::http::bigquery_client::test::create_client;
    use crate::http::bigquery_row_access_policy_client::BigqueryRowAccessPolicyClient;
    use crate::http::row_access_policy::list::ListRowAccessPoliciesRequest;
    use crate::http::table::get_iam_policy::GetIamPolicyRequest;
    use crate::http::table::TableReference;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn test_policy() {
        /*
        CREATE ROW ACCESS POLICY test_policy ON `rust_test_job.rust_test_load_result_iam` GRANT TO ('allAuthenticatedUsers') FILTER USING (string_field_1='test');
        CREATE ROW ACCESS POLICY test_policy2 ON `rust_test_job.rust_test_load_result_iam` GRANT TO ('allAuthenticatedUsers') FILTER USING (string_field_1='testhoge');
         */
        let (client, project) = create_client().await;
        let client = BigqueryRowAccessPolicyClient::new(Arc::new(client));

        let mut table1 = TableReference::default();
        table1.dataset_id = "rust_test_job".to_string();
        table1.project_id = project.to_string();
        table1.table_id = "rust_test_load_result_iam".to_string();

        // iam
        let policies = client
            .list(
                &table1.project_id,
                &table1.dataset_id,
                &table1.table_id,
                &ListRowAccessPoliciesRequest { page_size: Some(1) },
            )
            .await
            .unwrap();
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[0].filter_predicate, "string_field_1 = 'test'");
        assert_eq!(policies[1].filter_predicate, "string_field_1 = 'testhoge'");
        for p in policies {
            let r = p.row_access_policy_reference;
            let p = client
                .get_iam_policy(
                    &r.project_id,
                    &r.dataset_id,
                    &r.table_id,
                    &r.policy_id,
                    &GetIamPolicyRequest { options: None },
                )
                .await
                .unwrap();
            assert_eq!(p.bindings[0].role, "roles/bigquery.filteredDataViewer");
        }
    }
}
