use super::JsonpathMatch;
use serde::Serialize;
use serde_json::Value;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Jsonpath(String);
impl Deref for Jsonpath {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Jsonpath> for String {
    fn from(it: Jsonpath) -> Self {
        it.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize)]
pub struct MatchKey {
    path: Jsonpath,
}

#[derive(Debug, Clone, Hash, Serialize)]
pub struct MatchVal {
    path: Jsonpath,
    val: Value,
}

impl PartialEq<Self> for MatchVal {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for MatchVal {}

impl Ord for MatchVal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for MatchVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for JsonpathMatch {
    fn from(path: &str) -> Self {
        Self::Key(MatchKey { path: Jsonpath(path.to_string()) })
    }
}

impl From<String> for JsonpathMatch {
    fn from(path: String) -> Self {
        Self::Key(MatchKey { path: Jsonpath(path) })
    }
}

impl From<(&str, &Value)> for JsonpathMatch {
    fn from((path, val): (&str, &Value)) -> Self {
        Self::Val(MatchVal { path: Jsonpath(path.to_string()), val: val.to_owned() })
    }
}

impl JsonpathMatch {
    pub fn jsonpath(&self) -> &Jsonpath {
        match self {
            JsonpathMatch::Key(it) => &it.path,
            JsonpathMatch::Val(it) => &it.path,
        }
    }

    pub fn take_jsonpath(self) -> Jsonpath {
        match self {
            JsonpathMatch::Key(it) => it.path,
            JsonpathMatch::Val(it) => it.path,
        }
    }
}
