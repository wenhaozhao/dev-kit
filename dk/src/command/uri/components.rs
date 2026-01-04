use crate::command::uri::{QueryPartName, QueryPartVal, UriComponent, UriComponentValue};
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

impl FromStr for UriComponent {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();
        Ok(match value.as_str() {
            "scheme" => UriComponent::Scheme,
            "authority" => UriComponent::Authority,
            "host" => UriComponent::Host,
            "port" => UriComponent::Port,
            "path" => UriComponent::Path,
            value => {
                match &value[..1] {
                    "?" => {
                        UriComponent::Query(Some(QueryPartName::from_str(&value[1..])?))
                    }
                    _ => {
                        UriComponent::Query(Some(QueryPartName::from_str(value)?))
                    }
                }
            }
        })
    }
}

impl UriComponentValue {
    pub fn name(&self) -> &'static str {
        match self {
            UriComponentValue::Scheme(_) => "scheme",
            UriComponentValue::Authority(_) => "authority",
            UriComponentValue::Host(_) => "host",
            UriComponentValue::Port(_) => "port",
            UriComponentValue::Path(_) => "path",
            UriComponentValue::Query(_) => "query",
        }
    }

    pub fn string_value(&self) -> String {
        match self {
            UriComponentValue::Scheme(val) => val.to_string(),
            UriComponentValue::Authority(val) => val.to_string(),
            UriComponentValue::Host(val) => val.to_string(),
            UriComponentValue::Port(val) => val.to_string(),
            UriComponentValue::Path(val) => val.to_string(),
            UriComponentValue::Query(val) => {
                val.into_iter().map(|(k, v)| format!("{}={}", k, v)).join("&")
            }
        }
    }
}

impl QueryPartVal {
    pub fn concat(&self, other: &Self) -> Self {
        let vec = vec![self.clone(), other.clone()].into_iter().flat_map(|it| {
            match it {
                QueryPartVal::Single(Some(val)) => vec![val],
                QueryPartVal::Multi(vec) => vec,
                _ => vec![]
            }
        }).collect_vec();
        Self::Multi(vec)
    }
}

impl Display for QueryPartVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryPartVal::Single(string) => serde_json::to_string(string).unwrap_or_default().fmt(f),
            QueryPartVal::Multi(arr) => serde_json::to_string(arr).unwrap_or_default().fmt(f),
        }
    }
}