use google_cloud_auth::Project;

#[derive(Debug, Clone)]
pub enum ProjectOptions {
    Emulated(String),
    Project(Option<Project>),
}

impl ProjectOptions {
    pub fn new(emulator_host_var_name: &str) -> Self {
        std::env::var(emulator_host_var_name)
            .map(ProjectOptions::Emulated)
            .unwrap_or_else(|_| ProjectOptions::Project(None))
    }
}
