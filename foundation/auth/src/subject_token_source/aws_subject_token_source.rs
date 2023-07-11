use crate::credentials::CredentialSource;
use crate::error::Error;
use crate::subject_token_source::SubjectTokenSource;
use std::fmt::{Debug, Formatter};

pub struct AWSSubjectTokenSource {
    environment_id: String,
    region_url: String,
    region_cred_verification_url: String,
    cred_verification_url: String,
    target_resource: String,
    imdsv2_session_token_url: Option<String>,
}

impl TryFrom<&CredentialSource> for AWSSubjectTokenSource {
    type Error = Error;

    fn try_from(value: &CredentialSource) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl SubjectTokenSource for AWSSubjectTokenSource {
    async fn subject_token(&self) -> Result<String, Error> {
        todo!("implements")
    }
}
