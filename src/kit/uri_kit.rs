use anyhow::anyhow;
use derive_more::Display;
use percent_encoding::percent_decode_str;
use std::io::Read;

#[derive(Debug, Clone, Display)]
pub enum UriComponent {
    #[display("{_0}")]
    Url(url::Url),
    #[display("{_0}")]
    String(String),
}

impl TryFrom<String> for UriComponent {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            return Ok(UriComponent::String(string));
        }
        match url::Url::parse(&value) {
            Ok(url) => Ok(UriComponent::Url(url)),
            Err(_) => Ok(UriComponent::String(value)),
        }
    }
}
pub fn decode_uri_component<C>(uri_component: C) -> crate::Result<String>
where
    C: TryInto<UriComponent, Error = anyhow::Error>,
{
    let uri_component = uri_component.try_into()?;
    match uri_component {
        UriComponent::Url(url) => {
            Ok(percent_decode_str(url.as_str()).decode_utf8()?.to_string())
        }
        UriComponent::String(string) => {
            Ok(percent_decode_str(string.as_str()).decode_utf8()?.to_string())
        }
    }
}

