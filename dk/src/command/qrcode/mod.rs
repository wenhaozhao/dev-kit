use crate::command::qrcode::generator::QrCodeImage;
use crate::command::Command;
use derive_more::{Deref, FromStr};
use std::path::PathBuf;
use strum::Display;

#[derive(clap::Subcommand)]
pub enum QrCodeCommand {
    #[clap(about = "Generate a QR code, alias 'gen'", alias("gen"))]
    Generate {
        #[arg(help = "QR code content", default_value = "")]
        content: QrContent,
        #[arg(short, long, help = r#"
QR code error correction level, alias 'ecl'
    7%: Low error correction level, alias 'l'
    15%: Medium error correction level, alias 'm'
    25%: Quartile error correction level, alias 'q'
    30%: High error correction level, alias 'h'
        "#, alias = "ecl", default_value = "q")]
        ec_level: QrEcLevel,
        #[arg(short, long, help = r#"
QR code version,
    In QR code terminology, Version means the size of the generated image. Larger version means the size of code is larger, and therefore can carry more information.
    A normal QR code version. The parameter should be between 1 and 40.
    QR size: version * 4 + 17
"#, default_value = "3")]
        version: QrVersion,
        #[arg(short, long, help = "QR code output type, alias 'type'", alias = "type", default_value = "text")]
        output_type: OutputType,
        #[arg(short, long, help = "QR code output file")]
        file: Option<PathBuf>,
        #[arg(short, long, help = "plain text output")]
        plain: bool,
    }
}

#[derive(Debug, Clone, Deref)]
pub struct QrContent(String);

#[derive(Debug, Copy, Clone, Deref)]
pub struct QrEcLevel(qrcode::EcLevel);
#[derive(Debug, Copy, Clone, Deref)]
pub struct QrVersion(qrcode::Version);

#[derive(Debug, Copy, Clone, FromStr, Default, Display)]
pub enum OutputType {
    #[default]
    Text,
    Image,
    Svg,
}

impl Command for QrCodeCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            QrCodeCommand::Generate { content, ec_level, version, output_type, file, plain } => {
                let result = generator::generate(content, *ec_level, *version, *output_type);
                if !plain {
                    print!(r#"
Generate QR Code
Error correction level: {}
Version: {}
Output Type: {}
"#, ec_level, version, output_type);
                }
                match result {
                    Ok(QrCodeImage::Text(text)) => {
                        if let Some(file) = file {
                            match std::fs::write(file, text) {
                                Ok(_) => {
                                    if !plain {
                                        print!("Write QR Code to ")
                                    }
                                    println!("{}", file.display())
                                }
                                Err(err) => eprintln!("{}", err)
                            }
                        } else {
                            println!("{}", text);
                        }
                        Ok(())
                    }
                    Ok(QrCodeImage::Image(path)) | Ok(QrCodeImage::Svg(path)) => {
                        match if let Some(file_path) = file {
                            std::fs::copy(path, file_path).map(|_| file_path)
                        } else {
                            Ok(&path)
                        } {
                            Ok(path) => {
                                if !plain {
                                    print!("Write QR Code to ")
                                }
                                println!("{}", path.display())
                            }
                            Err(err) => {
                                eprintln!("{}", err)
                            }
                        }
                        Ok(())
                    }
                    Err(err) => {
                        eprintln!("Generate QR Code failed, {}", err);
                        Ok(())
                    }
                }
            }
        }
    }
}

pub mod generator;