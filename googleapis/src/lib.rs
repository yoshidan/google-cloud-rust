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

#[path = ""]
pub mod pubsub {
    #[path = "google.pubsub.v1.rs"]
    pub mod v1;
}
