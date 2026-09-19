use std::env::current_dir;

pub fn is_env_cog() -> bool {
    current_dir().unwrap().starts_with("/google/cog/cloud")
}
