use derive_more::with_trait::From;
use derive_more::Display;
use percent_encoding::percent_decode_str;

#[derive(Debug, Clone, Display, From)]
pub enum UriComponent {
    #[display("{_0}")]
    Url(url::Url),
    #[display("{_0}")]
    String(String),
}
pub fn decode_uri_component<C>(uri_component: C) -> crate::Result<String>
where
    C: Into<UriComponent>,
{
    let uri_component = uri_component.into();
    match uri_component {
        UriComponent::Url(url) => {
            Ok(percent_decode_str(url.as_str()).decode_utf8()?.to_string())
        }
        UriComponent::String(string) => {
            Ok(percent_decode_str(string.as_str()).decode_utf8()?.to_string())
        }
    }
}

