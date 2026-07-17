use crate::components::jsondiff::JsonDiffState;
use crate::components::jsonparser::JsonParserState;
use directories::ProjectDirs;

pub struct AppState {
    pub proj_dirs: ProjectDirs,
    pub jsonparser: JsonParserState,
    pub jsondiff: JsonDiffState,
}

impl AppState {
    pub fn init() -> Result<Self, String> {
        let proj_dirs = ProjectDirs::from("vip", "mhlife", "devkit")
            .ok_or("Failed to get project directories")?;
        let jsonparser = Self::init_jsonparser(&proj_dirs)?;
        let jsondiff = JsonDiffState::init()?;
        Ok(Self {
            proj_dirs,
            jsonparser,
            jsondiff,
        })
    }
}

impl AppState {
    fn init_jsonparser(proj_dirs: &ProjectDirs) -> Result<JsonParserState, String> {
        let path = proj_dirs.data_dir().join("jsonparser");
        if !path.exists() {
            let _ = std::fs::create_dir_all(&path).map_err(|e| e.to_string());
        }
        JsonParserState::init(path).map_err(|e| e.to_string())
    }
}
