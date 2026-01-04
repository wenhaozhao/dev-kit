use crate::command::uri::{QueryPartName, QueryPartVal, Uri, UriComponent, UriComponentValue};
use anyhow::anyhow;
use itertools::Itertools;
use percent_encoding::percent_decode_str;
use std::collections::BTreeMap;
use std::io::Read;
use std::str::FromStr;

impl FromStr for Uri {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            match string.trim() {
                "-" => Err(anyhow!("Not a valid input")),
                _ => Ok(Self::from_str(&string)?)
            }
        } else {
            match url::Url::parse(&value) {
                Ok(url) => Ok(Uri::Url(url)),
                Err(_) => Ok(Uri::String(value.to_string())),
            }
        }
    }
}

impl Uri {
    pub fn decode(&self) -> crate::Result<String> {
        match self {
            Uri::Url(url) => {
                Ok(percent_decode_str(url.as_str()).decode_utf8()?.to_string())
            }
            Uri::String(string) => {
                Ok(percent_decode_str(string.as_str()).decode_utf8()?.to_string())
            }
        }
    }

    pub fn encode(&self) -> crate::Result<String> {
        use percent_encoding::NON_ALPHANUMERIC as ascii_set;
        match self {
            Uri::Url(url) => {
                Ok(percent_encoding::utf8_percent_encode(url.as_str(), ascii_set).to_string())
            }
            Uri::String(string) => {
                Ok(percent_encoding::utf8_percent_encode(string.as_str(), ascii_set).to_string())
            }
        }
    }

    pub fn parse(&self, filter: &Option<Vec<UriComponent>>) -> crate::Result<Vec<UriComponentValue>> {
        let url = url::Url::try_from(self)?;
        let component_values = {
            let schema = url.scheme().to_lowercase().to_string();
            let components = vec![
                UriComponentValue::Scheme(schema.clone()),
                UriComponentValue::Authority(url.authority().to_string()),
                UriComponentValue::Host(url.host_str().unwrap_or_default().to_string()),
                UriComponentValue::Port({
                    url.port().unwrap_or_else(|| {
                        match schema.as_str() {
                            "http" => 80,
                            "https" => 443,
                            _ => 0
                        }
                    })
                }),
                UriComponentValue::Path(url.path().to_string()),
                UriComponentValue::Query({
                    let vals = url.query().and_then(|q|
                        serde_urlencoded::from_str::<Vec<(String, String)>>(q).ok()
                    ).unwrap_or_default();
                    let mut map = BTreeMap::<QueryPartName, QueryPartVal>::new();
                    for (name, value) in vals {
                        let name = QueryPartName(name.trim().to_string());
                        let value = QueryPartVal::Single(Some(value));
                        if let Some(exist) = map.get_mut(&name) {
                            *exist = exist.concat(&value);
                        } else {
                            map.insert(name, value);
                        }
                    }
                    map
                })
            ];
            components
        };
        let result = if let Some(filter) = &filter {
            component_values.into_iter().flat_map(|value| {
                match value {
                    UriComponentValue::Scheme(_) => {
                        if filter.iter().any(|it| if let UriComponent::Scheme = it { true } else { false }) {
                            Some(value)
                        } else {
                            None
                        }
                    }
                    UriComponentValue::Authority(_) => {
                        if filter.iter().any(|it| if let UriComponent::Authority = it { true } else { false }) {
                            Some(value)
                        } else {
                            None
                        }
                    }
                    UriComponentValue::Host(_) => {
                        if filter.iter().any(|it| if let UriComponent::Host = it { true } else { false }) {
                            Some(value)
                        } else {
                            None
                        }
                    }
                    UriComponentValue::Port(_) => {
                        if filter.iter().any(|it| if let UriComponent::Port = it { true } else { false }) {
                            Some(value)
                        } else {
                            None
                        }
                    }
                    UriComponentValue::Path(_) => {
                        if filter.iter().any(|it| if let UriComponent::Path = it { true } else { false }) {
                            Some(value)
                        } else {
                            None
                        }
                    }
                    UriComponentValue::Query(parts) => {
                        let filter = filter.iter().filter(|&it| {
                            if let UriComponent::Query(_) = it { true } else { false }
                        }).collect_vec();
                        if filter.is_empty() {
                            None
                        } else {
                            let parts = parts.into_iter().filter(|(k, _)| {
                                filter.iter().any(|&filter| {
                                    match filter {
                                        UriComponent::Query(Some(filter)) => {
                                            k.eq_ignore_ascii_case(filter)
                                        }
                                        UriComponent::Query(None) => {
                                            true
                                        }
                                        _ => unreachable!()
                                    }
                                })
                            }).collect_vec();
                            return Some(UriComponentValue::Query(parts.into_iter().collect::<BTreeMap<_, _>>()));
                        }
                    }
                }
            }).collect_vec()
        } else {
            component_values
        };
        Ok(result)
    }
}

impl TryFrom<&Uri> for url::Url {
    type Error = anyhow::Error;

    fn try_from(uri: &Uri) -> Result<Self, Self::Error> {
        match uri {
            Uri::Url(url) => Ok(url.clone()),
            Uri::String(string) => Ok(url::Url::from_str(string)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decode() {
        let uri = Uri::from_str("passport%3Dabc%40sohu%2Dinc%2Ecom").unwrap();
        assert_eq!(uri.decode().unwrap(), "passport=abc@sohu-inc.com");
    }
    #[test]
    fn test_encode() {
        let uri = Uri::from_str("passport=abc@sohu-inc.com").unwrap();
        assert_eq!(uri.encode().unwrap(), "passport%3Dabc%40sohu%2Dinc%2Ecom");
    }
    #[test]
    fn test_parse() {
        let uri = Uri::from_str("https://abc:123@sohu.com/a/b/c?q1=1&q2=a&q2=b").unwrap();
        let components = uri.parse(&None).unwrap();
        for component in &components {
            println!("{}=> {}", component.name(), component.string_value());
        }
        assert_eq!(components.len(), 6);
        //assert_eq!(components[0].value, "https");
    }

    #[test]
    fn test_parse_0() {
        let uri = Uri::from_str("https://abc:123@sohu.com/a/b/c?q1=1&q2=a&q2=b").unwrap();
        let components = uri.parse(
            &Some(vec!["scheme", "host", "port", "?q1", "q2"].into_iter().flat_map(|it| UriComponent::from_str(it).ok()).collect_vec())
        ).unwrap();
        for component in &components {
            println!("{}=> {}", component.name(), component.string_value());
        }
        assert_eq!(components.len(), 4);
    }
}
