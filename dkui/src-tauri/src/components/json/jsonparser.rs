use std::path::PathBuf;
use anyhow::anyhow;

pub type Result<T> = anyhow::Result<T>;

pub struct JsonParserState {
    active_tab_index: usize,
    tabs: Vec<JsonParserTab>,
}

pub struct JsonParserTab {
    json_input: JsonInput,
    json_output: JsonOutput,
    json_query: JsonQuery,
    selected_index: isize,
}

pub struct JsonInput(PathBuf);
pub struct JsonOutput(PathBuf);
pub struct JsonQuery(String);

impl JsonParserState {



}
