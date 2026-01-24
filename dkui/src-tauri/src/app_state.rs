use crate::components::JsonCache;

pub struct AppState {
    pub proj_dirs: directories::ProjectDirs,
    pub json_cache: JsonCache,
}

impl AppState {
    pub fn init() -> Result<Self, String> {
        let proj_dirs = directories::ProjectDirs::from("vip", "mhlife", "devkit").ok_or(
            "Failed to get project directories"
        )?;
        let json_cache = JsonCache::default();
        Ok(Self { proj_dirs, json_cache })
    }
}
