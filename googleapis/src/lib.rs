#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::large_enum_variant)]

#[path = "google.rpc.rs"]
pub mod rpc;

#[path = ""]
pub mod iam {
    #[path = "google.iam.v1.rs"]
    pub mod v1;
}

#[path = "google.longrunning.rs"]
pub mod longrunning;

#[path = "google.r#type.rs"]
pub mod r#type;

#[cfg(feature = "spanner")]
#[path = ""]
pub mod spanner {
    #[path = "google.spanner.v1.rs"]
    pub mod v1;

    #[path = ""]
    pub mod admin {

        #[path = ""]
        pub mod database {
            #[path = "google.spanner.admin.database.v1.rs"]
            pub mod v1;
        }
        #[path = ""]
        pub mod instance {
            #[path = "google.spanner.admin.instance.v1.rs"]
            pub mod v1;
        }
    }
}

#[cfg(feature = "pubsub")]
#[path = ""]
pub mod pubsub {
    #[cfg(not(feature = "bytes"))]
    #[path = "google.pubsub.v1.rs"]
    pub mod v1;

    #[cfg(feature = "bytes")]
    #[path = "bytes/google.pubsub.v1.rs"]
    pub mod v1;
}

#[cfg(feature = "storage")]
#[path = ""]
pub mod storage {
    #[path = "google.storage.v2.rs"]
    pub mod v2;
}

#[path = ""]
pub mod devtools {
    #[cfg(feature = "artifact-registry")]
    #[path = ""]
    pub mod artifact_registry {
        #[path = "google.devtools.artifactregistry.v1.rs"]
        pub mod v1;
    }
}

#[path = ""]
pub mod cloud {
    #[cfg(feature = "bigquery")]
    #[path = ""]
    pub mod bigquery {
        #[path = ""]
        pub mod storage {
            #[path = "google.cloud.bigquery.storage.v1.rs"]
            pub mod v1;
        }
    }

    #[cfg(feature = "kms")]
    #[path = ""]
    pub mod kms {
        #[path = "google.cloud.kms.v1.rs"]
        pub mod v1;
    }
}
