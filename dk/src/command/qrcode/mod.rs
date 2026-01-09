use crate::command::Command;
use crate::command::qrcode::generator::QrCodeImageVal;
use derive_more::{Deref, FromStr};
use qrcode::Version;
use std::ops::Deref;
use std::path::PathBuf;
use strum::Display;

#[derive(clap::Args)]
pub struct QrCodeArgs {
    #[arg(help = "QR code content", default_value = "")]
    content: QrContent,
    #[arg(
        short,
        long,
        help = r#"
    QR code error correction level, alias 'ecl'
        7%: Low error correction level, alias 'l'
        15%: Medium error correction level, alias 'm'
        25%: Quartile error correction level, alias 'q'
        30%: High error correction level, alias 'h'
            "#,
        alias = "ecl",
        default_value = "q"
    )]
    ec_level: QrEcLevel,
    #[arg(
        short,
        long,
        help = r#"
    QR code version,
        In QR code terminology, Version means the size of the generated image. Larger version means the size of code is larger, and therefore can carry more information.
        A normal QR code version. The parameter should be between 1 and 40.
        QR size: version * 4 + 17
    "#,
        default_value = "auto"
    )]
    version: QrVersion,
    #[arg(
        short,
        long,
        help = "QR code output type, alias 'type'",
        alias = "type",
        default_value = "text"
    )]
    output_type: OutputType,
    #[arg(short, long, help = "QR code output file")]
    file: Option<PathBuf>,
    #[arg(short, long, help = "plain text output")]
    plain: bool,
}

#[derive(Debug, Clone, Deref)]
pub struct QrContent(String);

#[derive(Debug, Copy, Clone, Deref)]
pub struct QrEcLevel(qrcode::EcLevel);

#[derive(Debug, Copy, Clone)]
pub enum QrVersion {
    Auto,
    Version(Version),
}

#[derive(Debug, Copy, Clone, FromStr, Default, Display)]
pub enum OutputType {
    #[default]
    Text,
    Image,
    Svg,
}

impl Command for QrCodeArgs {
    fn run(&self) -> crate::Result<()> {
        let Self {
            content,
            ec_level,
            version,
            output_type,
            file,
            plain,
        } = self;
        let result = generator::generate(content, ec_level, version, *output_type);
        match result {
            Ok(result) => {
                let show_detail = !plain;
                if show_detail {
                    print!(
                        r#"
Generate QR Code
Error correction level: {}
Version: {}
Output Type: {}
"#,
                        result.ec_level,
                        result.version,
                        result.out_put_type()
                    );
                }
                match result.deref() {
                    QrCodeImageVal::Text(text) => {
                        if let Some(file) = file {
                            match std::fs::write(file, text) {
                                Ok(_) => {
                                    if show_detail {
                                        print!("Write QR Code to ")
                                    }
                                    println!("{}", file.display())
                                }
                                Err(err) => eprintln!("{}", err),
                            }
                        } else {
                            println!("{}", text);
                        }
                        Ok(())
                    }
                    QrCodeImageVal::Image(path) | QrCodeImageVal::Svg(path) => {
                        match if let Some(file_path) = file {
                            std::fs::copy(path, file_path).map(|_| file_path)
                        } else {
                            Ok(path)
                        } {
                            Ok(path) => {
                                if show_detail {
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
                }
            }
            Err(err) => {
                eprintln!("Generate QR Code failed, {}", err);
                Ok(())
            }
        }
    }
}

pub mod generator;
