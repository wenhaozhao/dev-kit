use super::{DiffTool, Json, KeyPatternType, QueryType};
use itertools::Itertools;
use jsonpath_rust::JsonPath;
use serde::ser::Error;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;
use std::{env, fs};

impl Json {
    pub fn search_paths(&self, query: Option<&str>, query_type: Option<QueryType>) -> crate::Result<Vec<Jsonpath>> {
        let json = Arc::<Value>::try_from(self)?;
        let query = query.map(|it| it.trim()).filter(|it| it.len() > 0).unwrap_or("");
        let (kp, qt) = Self::parse_query_type(query, query_type)?;
        match (kp, qt, query, query.is_empty()) {
            (_, _, _, true) => {
                Ok(vec![])
            }
            (Some(key_pattern), _, _, false) => {
                Ok(Self::search_key_actual(&json, &key_pattern, None))
            }
            (None, Some(QueryType::JsonPath), query, false) => {
                let (prefix, keyword) = if let Some((prefix, keyword)) = query.rsplit_once(".") {
                    (prefix, Some(keyword))
                } else {
                    (query, None)
                };
                match (prefix, keyword, keyword.unwrap_or("").is_empty()){
                    (prefix, Some(keyword), false) => {
                        Ok(Self::search_key_actual(&json, &KeyPattern::Prefix(keyword.to_lowercase()), Some(prefix)))
                    }
                    (prefix,_,_) => {
                        Ok(json.query(prefix)?.iter().flat_map(|it| match it {
                            Value::Object(map) => map.keys().map(|k| k.to_string()).collect_vec(),
                            Value::Array(_) => vec!["*".to_string()],
                            _ => vec![],
                        }).unique().map(|it| Jsonpath(it)).collect_vec())
                    }
                }
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn query(&self, query: Option<&str>, query_type: Option<QueryType>, beauty: bool) -> crate::Result<String> {
        let json = Arc::<Value>::try_from(self)?;
        let query_vals = Self::query_actual(&json, query, query_type)?;
        if beauty {
            Ok(serde_json::to_string_pretty(&query_vals)?)
        } else {
            Ok(serde_json::to_string(&query_vals)?)
        }
    }

    pub fn diff(
        &self,
        other: &Self,
        query: Option<&str>,
        query_type: Option<QueryType>,
        diff_tool: Option<DiffTool>,
    ) -> crate::Result<()> {
        let tmp_dir = env::temp_dir()
            .join("jsondiff")
            .join(chrono::Local::now().format("%Y%m%d%H%M%S%f").to_string());
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        let left = self;
        let right = other;
        let _ = fs::create_dir_all(&tmp_dir)?;
        let left = left.diff_prepare(query.as_deref(), query_type)?;
        let left_path = tmp_dir.join("left.json");
        fs::write(&left_path, left)?;
        println!("write left to file {}", left_path.display());
        let right = right.diff_prepare(query.as_deref(), query_type)?;
        let right_path = tmp_dir.join("right.json");
        fs::write(&right_path, right)?;
        println!("write right to file {}", right_path.display());
        let diff_tool = diff_tool.unwrap_or_default();
        if diff_tool.is_available() {
            println!("diff with {}", diff_tool);
            diff_tool.diff(&left_path, &right_path)?;
        } else {
            eprintln!("diff tool {} is not installed", diff_tool);
            println!(
                r#"
install {} command-line interface, see:
{}"#,
                diff_tool,
                diff_tool.how_to_install()
            )
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum QueryVals {
    Origin(Value),
    JsonPath(Vec<Value>),
    KeyPattern(BTreeMap<Jsonpath, Vec<Value>>),
}

impl serde::Serialize for QueryVals {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            QueryVals::Origin(vals) => {
                vals.serialize(serializer)
            }
            QueryVals::JsonPath(vals) => {
                match serde_json::to_value(vals) {
                    Ok(v) => v.serialize(serializer),
                    Err(err) => Err(S::Error::custom(format!("{}", err)))
                }
            }
            QueryVals::KeyPattern(vals) => {
                match serde_json::to_value(vals) {
                    Ok(v) => v.serialize(serializer),
                    Err(err) => Err(S::Error::custom(format!("{}", err)))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum KeyPattern {
    Prefix(String),
    Suffix(String),
    Contains(String),
    Regex(regex::Regex),
}

impl From<&KeyPattern> for KeyPatternType {
    fn from(value: &KeyPattern) -> Self {
        match value {
            KeyPattern::Prefix(_) => KeyPatternType::Prefix,
            KeyPattern::Suffix(_) => KeyPatternType::Suffix,
            KeyPattern::Contains(_) => KeyPatternType::Contains,
            KeyPattern::Regex(_) => KeyPatternType::Regex,
        }
    }
}

impl KeyPattern {
    fn new(key_pattern: &str, pattern_type: KeyPatternType) -> crate::Result<Self> {
        match pattern_type {
            KeyPatternType::Prefix => Ok(Self::Prefix(key_pattern.to_lowercase())),
            KeyPatternType::Suffix => Ok(Self::Suffix(key_pattern.to_lowercase())),
            KeyPatternType::Contains => Ok(Self::Contains(key_pattern.to_lowercase())),
            KeyPatternType::Regex => Ok(Self::Regex(
                regex::RegexBuilder::new(key_pattern).case_insensitive(true).build()?
            )),
        }
    }

    fn guess(key_pattern: &str) -> crate::Result<Self> {
        match regex::RegexBuilder::new(key_pattern).case_insensitive(true).build() {
            Ok(regex) => {
                Ok(Self::Regex(regex))
            }
            Err(_) => {
                Self::new(key_pattern, KeyPatternType::default())
            }
        }
    }

    fn match_key(&self, key: &str) -> bool {
        match self {
            KeyPattern::Prefix(prefix) => {
                let key = key.to_lowercase();
                key.starts_with(prefix)
            }
            KeyPattern::Suffix(suffix) => {
                let key = key.to_lowercase();
                key.ends_with(suffix)
            }
            KeyPattern::Contains(contains) => {
                let key = key.to_lowercase();
                key.contains(contains)
            }
            KeyPattern::Regex(regex) => {
                regex.is_match(key)
            }
        }
    }
}

impl Json {
    fn parse_query_type(
        query: &str,
        query_type: Option<QueryType>,
    ) -> crate::Result<(Option<KeyPattern>, Option<QueryType>)> {
        match (query_type, query.is_empty(), query.starts_with("$")) {
            (Some(QueryType::JsonPath), _, _) | (None, false, true) => {
                Ok((None, Some(QueryType::JsonPath)))
            }
            (Some(qt @ QueryType::KeyPattern(kpt)), _, _) => {
                Ok((Some(KeyPattern::new(query, kpt)?), Some(qt)))
            }
            (None, true, _) => {
                Ok((None, None))
            }
            (None, false, false) => {
                let kp = KeyPattern::guess(query).ok();
                let qt = kp.as_ref().map(|it|
                    KeyPatternType::from(it)
                ).map(|kpt|
                    QueryType::KeyPattern(kpt)
                );
                Ok((kp, qt))
            }
        }
    }

    fn query_actual(
        json: &Value,
        query: Option<&str>,
        query_type: Option<QueryType>,
    ) -> crate::Result<QueryVals> {
        let query = query.map(|it| it.trim()).filter(|it| it.len() > 0).unwrap_or("");
        let (kp, qt) = Self::parse_query_type(query, query_type)?;
        match (kp, qt, query, query.is_empty()) {
            (_, _, _, true) => {
                Ok(QueryVals::Origin(json.to_owned()))
            }
            (Some(key_pattern), _, _, false) => {
                let json_paths = Self::search_key_actual(&json, &key_pattern, None);
                let mut map = BTreeMap::new();
                for path in json_paths.into_iter() {
                    let arr = json.query(&path).into_iter().flatten().map(|it| it.to_owned()).collect_vec();
                    let _ = map.insert(path, arr);
                }
                Ok(QueryVals::KeyPattern(map))
            }
            (None, Some(QueryType::JsonPath), query, false) => {
                let query = query.trim_end_matches(".");
                let arr = json.query(&query).into_iter().flatten().map(|it| it.to_owned()).collect_vec();
                Ok(QueryVals::JsonPath(arr))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn diff_prepare(&self, query: Option<&str>, query_type: Option<QueryType>) -> crate::Result<String> {
        let json = Arc::<Value>::try_from(self)?;
        let array = Self::query_actual(&json, query, query_type)?;
        let pretty = serde_json::to_string_pretty(&array)?;
        Ok(pretty)
    }

    fn search_key_actual(json: &Value, key_pattern: &KeyPattern, prefix: Option<&str>) -> Vec<Jsonpath> {
        let jsons = match &prefix {
            Some(prefix) => {
                match json.query(prefix) {
                    Ok(arr) => arr,
                    Err(_) => vec![json],
                }
            }
            None => vec![json],
        };
        Self::search_key_recursive(&jsons, &key_pattern, prefix.unwrap_or("$")).into_iter()
            .unique().map(|it| Jsonpath(it)).collect_vec()
    }
}

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

impl Json {
    fn search_key_recursive(jsons: &[&Value], key_pattern: &KeyPattern, path: &str) -> Vec<String> {
        jsons.iter().flat_map(|&json| {
            match json {
                Value::Object(map) => {
                    let mut vec = Vec::with_capacity(map.len());
                    for (k, v) in map {
                        let path = format!("{}.{}", path, k);
                        let mut children = Self::search_key_recursive(&vec![v], key_pattern, &path);
                        if key_pattern.match_key(k) {
                            vec.push(path);
                        }
                        let _ = vec.append(&mut children);
                    }
                    vec
                }
                Value::Array(array) => {
                    let mut vec = Vec::with_capacity(array.len());
                    let path = format!("{}.*", path);
                    let jsons = array.iter().map(|it| it).collect_vec();
                    let mut children = Self::search_key_recursive(&jsons, key_pattern, &path);
                    if children.len() > 0 {
                        let _ = vec.append(&mut children);
                    }
                    vec
                }
                _ => {
                    vec![]
                }
            }
        }).collect_vec()
    }
}