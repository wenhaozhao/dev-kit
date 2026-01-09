use crate::command::qrcode::{OutputType, QrContent, QrEcLevel, QrVersion};
use derive_more::Deref;
use image::Luma;
use qrcode::render::{svg, unicode};
use qrcode::types::QrError;
use qrcode::{EcLevel, QrCode, Version};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

pub fn generate<'a>(
    content: &'a QrContent,
    ec_level: &'a QrEcLevel,
    version: &'a QrVersion,
    output_type: OutputType,
) -> crate::Result<QrCodeImage<'a>> {
    let (qr_code, version) = create_qr_code(content, version, ec_level)?;
    let image = match output_type {
        OutputType::Text => {
            let image = qr_code
                .render::<unicode::Dense1x2>()
                .dark_color(unicode::Dense1x2::Light)
                .light_color(unicode::Dense1x2::Dark)
                .build();
            QrCodeImageVal::Text(image)
        }
        OutputType::Image => {
            let image = {
                let image = qr_code.render::<Luma<u8>>().build();
                let path =
                    std::env::temp_dir().join(format!("qrcode-{}.png", uuid::Uuid::new_v4()));
                let _ = image.save(&path)?;
                path
            };
            QrCodeImageVal::Image(image)
        }
        OutputType::Svg => {
            let image = {
                let image = qr_code
                    .render()
                    .min_dimensions(200, 200)
                    .dark_color(svg::Color("#800000"))
                    .light_color(svg::Color("#ffff80"))
                    .build();
                let path =
                    std::env::temp_dir().join(format!("qrcode-{}.svg", uuid::Uuid::new_v4()));
                let _ = std::fs::write(&path, image.as_bytes())?;
                path
            };
            QrCodeImageVal::Svg(image)
        }
    };
    Ok(QrCodeImage {
        content,
        ec_level: *ec_level,
        version,
        image,
    })
}

fn create_qr_code(
    content_str: &str,
    qr_version: &QrVersion,
    qr_ec_level: &QrEcLevel,
) -> Result<(QrCode, QrVersion), QrError> {
    let mut version = match qr_version {
        QrVersion::Auto => Version::Normal(3),
        QrVersion::Version(val) => *val,
    };
    let ec_level = **qr_ec_level;
    loop {
        match QrCode::with_version(content_str, version, ec_level) {
            Ok(val) => {
                return Ok((val, QrVersion::Version(version)));
            }
            Err(QrError::DataTooLong) => match qr_version {
                QrVersion::Auto => {
                    version = match version {
                        Version::Normal(val) => Version::Normal(val + 1),
                        Version::Micro(val) => Version::Micro(val + 1),
                    };
                    continue;
                }
                _ => return Err(QrError::DataTooLong),
            },
            Err(QrError::InvalidVersion) => {
                return Err(QrError::DataTooLong);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}

impl FromStr for QrEcLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        Ok(match s.as_str() {
            "0" | "7" | "7%" | "l" => Self(EcLevel::L),
            "1" | "15" | "15%" | "m" => Self(EcLevel::M),
            "2" | "25" | "25%" | "q" => Self(EcLevel::Q),
            "3" | "30" | "30%" | "h" => Self(EcLevel::H),
            _ => Self::default(),
        })
    }
}

impl Display for QrEcLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.deref() {
            EcLevel::L => write!(f, "7%"),
            EcLevel::M => write!(f, "15%"),
            EcLevel::Q => write!(f, "25%"),
            EcLevel::H => write!(f, "30%"),
        }
    }
}

impl Default for QrEcLevel {
    fn default() -> Self {
        Self(EcLevel::Q)
    }
}

impl FromStr for QrVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "auto" => Ok(Self::Auto),
            val => Ok(val
                .parse::<u8>()
                .map(|it| Self::Version(Version::Normal(it as i16)))
                .unwrap_or_default()),
        }
    }
}

impl Display for QrVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Version(val @ Version::Normal(int_val))
            | Self::Version(val @ Version::Micro(int_val)) => {
                write!(f, "{} ({}*{})", int_val, val.width(), val.width())
            }
        }
    }
}

impl Default for QrVersion {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Deref)]
pub struct QrCodeImage<'a> {
    pub content: &'a QrContent,
    pub ec_level: QrEcLevel,
    pub version: QrVersion,
    #[deref]
    pub image: QrCodeImageVal,
}
#[derive(Debug, Clone)]
pub enum QrCodeImageVal {
    Text(String),
    Image(PathBuf),
    Svg(PathBuf),
}

impl Display for QrCodeImage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Display;
        match &self.image {
            QrCodeImageVal::Text(image) => Display::fmt(image, f),
            QrCodeImageVal::Image(image) => Display::fmt(&image.display(), f),
            QrCodeImageVal::Svg(image) => Display::fmt(&image.display(), f),
        }
    }
}

impl QrCodeImage<'_> {
    pub fn out_put_type(&self) -> OutputType {
        OutputType::from(self.deref())
    }
}

impl From<&QrCodeImageVal> for OutputType {
    fn from(value: &QrCodeImageVal) -> Self {
        match value {
            QrCodeImageVal::Text(_) => OutputType::Text,
            QrCodeImageVal::Image(_) => OutputType::Image,
            QrCodeImageVal::Svg(_) => OutputType::Svg,
        }
    }
}
