use anyhow::anyhow;
use derive_more::{Deref, Display};
use itertools::Itertools;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Method;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, mem};

#[derive(Debug, Clone, Deref, Display)]
pub struct JetBrainsHttp(Arc<ParseBuffer>);

const COMMENT_SYMBOL: &str = "#";

lazy_static! {
    static ref grammar_pattern_comments: regex::Regex = regex::Regex::new(r"^[\s\t]*#+.*").unwrap();
}

lazy_static! {
    /// 新请求标记
    static ref grammar_pattern_start: regex::Regex = regex::Regex::new(r"^###+.*$").unwrap();
}

lazy_static! {
    static ref grammar_pattern_url: regex::Regex = {
        regex::RegexBuilder::new(r"^([A-Z]+)\s+(http[s]?.*)$")
        .case_insensitive(true)
        .build().unwrap()
    };
}

lazy_static! {
    static ref grammar_pattern_url_default: regex::Regex = {
        regex::RegexBuilder::new(r"^(http[s]?.*)$")
        .case_insensitive(true)
        .build().unwrap()
    };
}

lazy_static! {
    static ref grammar_pattern_url_parts: regex::Regex = regex::Regex::new(r"^[\s\t]+(.+)$").unwrap();
}

lazy_static! {
    static ref grammar_pattern_header: regex::Regex = regex::Regex::new(r"^([\w\d-]+)\s*:\s*(.*)$").unwrap();
}

impl JetBrainsHttp {
    fn try_parse(s: &str) -> crate::Result<Self> {
        let mut buffers = vec![];
        let mut buffer = ParseBuffer::default();
        let mut lines = s.lines().enumerate();
        enum ParseStep {
            Init,
            Start,
            Url,
            Header,
            Body,
        }
        let mut current_step = ParseStep::Init;
        'line_loop: loop {
            if let Some((_, line)) = lines.next() {
                loop {
                    match current_step {
                        ParseStep::Init => {
                            if line.trim().is_empty() {
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                if buffer.method.is_some() {
                                    let buffer = mem::take(&mut buffer);
                                    buffers.push(buffer);
                                }
                                current_step = ParseStep::Start;
                                continue;
                            } else if grammar_pattern_url.is_match(&line) {
                                current_step = ParseStep::Start;
                                continue;
                            }  else if grammar_pattern_url_default.is_match(&line) {
                                current_step = ParseStep::Start;
                                continue;
                            } else {
                                continue 'line_loop;
                            }
                        }
                        ParseStep::Start => {
                            if line.trim().is_empty() {
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                if buffer.name.is_none() {
                                    buffer.name = Some(line.replace(COMMENT_SYMBOL, ""));
                                }
                                continue 'line_loop;
                            } else if grammar_pattern_comments.is_match(&line) {
                                continue 'line_loop;
                            } else if grammar_pattern_url.is_match(&line) {
                                current_step = ParseStep::Url;
                                continue;
                            } else if grammar_pattern_url_default.is_match(&line) {
                                current_step = ParseStep::Url;
                                continue;
                            } else {
                                continue 'line_loop;
                            }
                        }
                        ParseStep::Url => {
                            if line.trim().is_empty() {
                                current_step = ParseStep::Body;
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                current_step = ParseStep::Init;
                                continue;
                            } else if grammar_pattern_comments.is_match(&line) {
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                current_step = ParseStep::Init;
                                continue;
                            } else if let Some((_, [method, url])) = grammar_pattern_url.captures(&line).map(|it|
                                it.extract()
                            ) {
                                buffer.method = Some(method.to_uppercase());
                                buffer.url_parts.push(url.trim().to_string());
                                continue 'line_loop;
                            } else if let Some((_, [url])) = grammar_pattern_url_default.captures(&line).map(|it|
                                it.extract()
                            ) {
                                buffer.method = Some(Method::GET.to_string());
                                buffer.url_parts.push(url.trim().to_string());
                                continue 'line_loop;
                            } else if let Some((_, [url_part])) = grammar_pattern_url_parts.captures(&line).map(|it| it.extract()) {
                                buffer.url_parts.push(url_part.trim().to_string());
                                continue 'line_loop;
                            } else if grammar_pattern_header.is_match(&line) {
                                current_step = ParseStep::Header;
                                continue;
                            } else {
                                continue 'line_loop;
                            }
                        }
                        ParseStep::Header => {
                            if line.trim().is_empty() {
                                current_step = ParseStep::Body;
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                current_step = ParseStep::Init;
                                continue;
                            } else if grammar_pattern_comments.is_match(&line) {
                                continue 'line_loop;
                            } else if let Some((_, [name, value])) = grammar_pattern_header.captures(&line).map(|it| it.extract()) {
                                buffer.headers.push((name.to_string(), value.to_string()));
                                current_step = ParseStep::Header;
                                continue 'line_loop;
                            } else {
                                continue 'line_loop;
                            }
                        }
                        ParseStep::Body => {
                            if line.trim().is_empty() {
                                continue 'line_loop;
                            } else if grammar_pattern_start.is_match(&line) {
                                current_step = ParseStep::Init;
                                continue;
                            } else if grammar_pattern_comments.is_match(&line) {
                                continue 'line_loop;
                            } else {
                                buffer.body.push(line.to_string());
                                continue 'line_loop;
                            }
                        }
                    }
                }
            } else {
                if buffer.method.is_some() {
                    buffers.push(buffer);
                }
                break;
            }
        }
        let buffer = buffers.into_iter().next().ok_or(anyhow!("Not a valid input"))?;
        Ok(JetBrainsHttp(Arc::new(buffer)))
    }
}


impl FromStr for JetBrainsHttp {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}


#[derive(Debug, Default, Clone)]
pub struct ParseBuffer {
    name: Option<String>,
    method: Option<String>,
    url_parts: Vec<String>,
    headers: Vec<(String, String)>,
    body: Vec<String>,
}

impl TryFrom<&ParseBuffer> for url::Url {
    type Error = anyhow::Error;

    fn try_from(value: &ParseBuffer) -> Result<Self, Self::Error> {
        let url_string = value.url_parts.iter().map(|it| it.trim()).join("");
        let url = url::Url::from_str(&url_string).map_err(|err| anyhow!("Invalid url: {}", err))?;
        Ok(url)
    }
}

impl TryFrom<&ParseBuffer> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &ParseBuffer) -> Result<Self, Self::Error> {
        Ok(value.method.as_ref().and_then(|it|
            Method::from_str(it).ok()
        ).unwrap_or(Method::GET))
    }
}

#[derive(Debug, Clone, Default, Deref)]
struct HeaderMap_(HeaderMap);

impl From<HeaderMap_> for HeaderMap {
    #[inline(always)]
    fn from(value: HeaderMap_) -> Self {
        value.0
    }
}
impl TryFrom<&ParseBuffer> for HeaderMap_ {
    type Error = anyhow::Error;

    fn try_from(value: &ParseBuffer) -> Result<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        for (k, v) in &value.headers {
            headers.insert(
                HeaderName::from_str(&k)?,
                v.parse()?,
            );
        }
        Ok(HeaderMap_(headers))
    }
}

impl Display for HeaderMap_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter().map(|(k, v)| {
            let k = k.as_str();
            let v = v.to_str().unwrap_or("");
            format!("{}: {}", k, v)
        }).join("\n").fmt(f)
    }
}

impl TryFrom<&ParseBuffer> for reqwest::Request {
    type Error = anyhow::Error;

    fn try_from(parse_buffer: &ParseBuffer) -> crate::Result<Self> {
        let method = parse_buffer.try_into()?;
        let url = parse_buffer.try_into()?;
        let mut req = reqwest::Request::new(method, url);
        *(req.headers_mut()) = HeaderMap_::try_from(parse_buffer)?.into();
        let body = parse_buffer.body.join("\n");
        *req.body_mut() = Some(reqwest::Body::from(body));
        Ok(req)
    }
}

impl Display for ParseBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(r#"
{} {}
{}

{}
        "#,
                                 Method::try_from(self).map_err(|_| fmt::Error)?,
                                 url::Url::try_from(self).map_err(|_| fmt::Error)?,
                                 HeaderMap_::try_from(self).map_err(|_| fmt::Error)?,
                                 self.body.join("\n")
        ))
    }
}


#[cfg(test)]
mod tests {
    use crate::command::http_parser::JetBrainsHttp;
    use std::str::FromStr;

    #[test]
    fn test_request_parse() {
        let string = r#"
GET https://api.store.sohu.com/android/v1/aggregation/app/commodity?privilege_id=3&types=1%2C2&
    gid=x010740210101a982cb8ac86e0006c081dfd59fb55bb&sver=10.1.90&ua=SohuVideoMobile%2F10.1.90%20(Platform%2F6)&
    buildNo=12101801&ssl=1&poid=1&
    uid=SV_WEPEkaJfcXy0pZU2bMY7LVnfS4GwN-y5aPEBCb1_luyOWaWhAtIyYvmP_7V6slFSU_KzUQuSZ0XV2UPYi3DlwEdBEAd8_fn8kpX8bApgj5Q&
    partner=93&passport=2003384673480785920%40sohu.com&api_key=9854b2afa779e1a6bff1962447a09dbd&appid=107402&
    abmode=1A_35B_91A_142A_152B_177A_202A_227A_262B_346B_353C_477A_484A_493A_529B_547A_574A_583A_592A_601A_610A_619B_628B_637A_646A_655A_664B_v99989_9998C_s45_t10000&
    plat=6&
    auth_token=eyJleHAiOjE3NzQyNTYwNjYwNDcsImlhdCI6MTc2NjQ4MDA2NjA0NywicHAiOiIyMDAzMzg0NjczNDgwNzg1OTIwQHNvaHUuY29tIiwidGsiOiIzYnA0NFMxRmR2dk5KU210MjJ0bUdsUWZWQnlVbnY1ZyIsInYiOjB9.Pp9h9RUdsT4ttKfbXFETe8Fk6MG2muvSBD5iQ01uOW0
Host: api.store.sohu.com
app_id: 1
appid: 107402
appvs: 10.1.90
gid: x010740210101a982cb8ac86e0006c081dfd59fb55bb
plat: 6
pn: 93
poid: 1
sver: 10.1.90
svsign: 5D4E555596172375A39EAD108858D484
sys: android
timestamp: 1766480896006
traceparent: 00-f548150502e3ad0d681e9d5f9ba9e33c-f548150502e3ad0d-01
ua: SohuVideoMobile/10.1.90 (Platform/6)
uid: SV_WEPEkaJfcXy0pZU2bMY7LVnfS4GwN-y5aPEBCb1_luyOWaWhAtIyYvmP_7V6slFSU_KzUQuSZ0XV2UPYi3DlwEdBEAd8_fn8kpX8bApgj5Q
user-agent: SohuVideoMobile/10.1.90 (Platform/6)
        "#;

        let target = JetBrainsHttp::from_str(string).unwrap();
        println!("{}", target);
    }
}