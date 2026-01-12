use crate::command::StringInput;
use base64::Engine;
use itertools::Itertools;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum Base64Command {
    #[clap(about = "base64 decode, alias 'd'", alias = "d")]
    Decode {
        #[arg(help = "base64 text to decode", default_value = "")]
        input: StringInput,
        #[arg(short, long, help = "url safe", default_value = "false")]
        url_safe: bool,
        #[arg(short, long, help = "no padding", default_value = "false")]
        no_pad: bool,
        #[arg(short, long, help = "raw output", default_value = "false")]
        raw_output: bool,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "base64 encode, alias 'e'", alias = "e")]
    Encode {
        #[arg(help = "base64 text to decode", default_value = "")]
        input: StringInput,
        #[arg(short, long, help = "url safe", default_value = "false")]
        url_safe: bool,
        #[arg(short, long, help = "no padding", default_value = "false")]
        no_pad: bool,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
}

pub fn decode(input: &str, url_safe: bool, no_pad: bool) -> crate::Result<Vec<u8>> {
    let text = if let Some(input) = PathBuf::from_str(input).ok().and_then(|p| fs::read_to_string(p).ok()) {
        input
    } else {
        input.to_string()
    };
    if url_safe {
        if no_pad {
            base64::prelude::BASE64_URL_SAFE.decode(text.as_bytes())
        } else {
            base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(text.as_bytes())
        }
    } else {
        if no_pad {
            base64::prelude::BASE64_STANDARD_NO_PAD.decode(text.as_bytes())
        } else {
            base64::prelude::BASE64_STANDARD.decode(text.as_bytes())
        }
    }.map_err(|e| anyhow::anyhow!("base64 encode failed: {}", e))
}

pub fn encode(input: &str, url_safe: bool, no_pad: bool) -> crate::Result<String> {
    let data = if let Some(input) = PathBuf::from_str(input).ok().and_then(|p| fs::read(p).ok()) {
        input
    } else {
        input.as_bytes().to_vec()
    };
    let string = if url_safe {
        if no_pad {
            base64::prelude::BASE64_URL_SAFE.encode(data)
        } else {
            base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(data)
        }
    } else {
        if no_pad {
            base64::prelude::BASE64_STANDARD_NO_PAD.encode(data)
        } else {
            base64::prelude::BASE64_STANDARD.encode(data)
        }
    };
    Ok(string)
}

pub enum Base64Val {
    EncodeString(String),
    DecodeBytes(Vec<u8>),
}

impl super::Command for Base64Command {
    fn run(&self) -> crate::Result<()> {
        match self {
            Self::Decode { input, url_safe, no_pad, raw_output, file } => {
                let data = decode(&input, *url_safe, *no_pad)?;
                match (*raw_output, file) {
                    (false, None) => {
                        let text = String::from_utf8_lossy(&data);
                        println!("{}", text);
                    }
                    (true, None) => {
                        let text = data.chunks(8).flatten().map(|it| {
                            format!("{:02x} ", *it)
                        }).join("\n");
                        println!("{}", text);
                    }
                    (false, Some(file)) => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        let _ = std::fs::write(file, text)?;
                        println!("write to {}", file.display())
                    }
                    (true, Some(file)) => {
                        let _ = std::fs::write(file, data)?;
                        println!("write to {}", file.display())
                    }
                }
            }
            Self::Encode { input, url_safe, no_pad, file } => {
                let text = encode(&input, *url_safe, *no_pad)?;
                if let Some(file) = file {
                    let _ = std::fs::write(file, text)?;
                    println!("write to {}", file.display())
                } else {
                    println!("{}", text);
                }
            }
        }
        Ok(())
    }
}


