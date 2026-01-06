use crate::command::qrcode::{OutputType, QrContent, QrEcLevel, QrVersion};
use crate::command::read_stdin;
use anyhow::anyhow;
use image::Luma;
use qrcode::render::{svg, unicode};
use qrcode::{EcLevel, QrCode, Version};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

pub fn generate<E: Deref<Target=EcLevel>, V: Deref<Target=Version>>(
    content: &QrContent,
    ec_level: E,
    version: V,
    output_type: OutputType,
) -> crate::Result<QrCodeImage> {
    let qr_code = QrCode::with_version(content.as_str(), *version, *ec_level)?;
    match output_type {
        OutputType::Text => {
            let image = qr_code.render::<unicode::Dense1x2>()
                .dark_color(unicode::Dense1x2::Light)
                .light_color(unicode::Dense1x2::Dark)
                .build();
            Ok(QrCodeImage::Text(image))
        }
        OutputType::Image => {
            let image = qr_code.render::<Luma<u8>>().build();
            let tmp_path = std::env::temp_dir().join(format!("qrcode-{}.png", uuid::Uuid::new_v4()));
            let _ = image.save(&tmp_path)?;
            Ok(QrCodeImage::Image(tmp_path))
        }
        OutputType::Svg => {
            let image = qr_code.render()
                .min_dimensions(200, 200)
                .dark_color(svg::Color("#800000"))
                .light_color(svg::Color("#ffff80"))
                .build();
            let tmp_path = std::env::temp_dir().join(format!("qrcode-{}.svg", uuid::Uuid::new_v4()));
            let _ = std::fs::write(&tmp_path, image.as_bytes())?;
            Ok(QrCodeImage::Svg(tmp_path))
        }
    }
}

impl FromStr for QrContent {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let input = read_stdin().unwrap_or(value.to_string());
        if input.is_empty() {
            Err(anyhow!("input is empty"))
        } else {
            Ok(Self(input))
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
            _ => Self::default()
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
        Ok(s.parse::<u8>().map(|it|
            Self(Version::Normal(it as i16))
        ).unwrap_or_default())
    }
}

impl Display for QrVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}*{}", self.width(), self.width()))
    }
}

impl Default for QrVersion {
    fn default() -> Self {
        Self(Version::Normal(3))
    }
}

pub enum QrCodeImage {
    Text(String),
    Image(PathBuf),
    Svg(PathBuf),
}