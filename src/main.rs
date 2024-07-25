use std::fs::File;
use std::io::Read;
use clap::{Args, Parser};
use once_cell::sync::Lazy;
use serde_json::{Value};
use windows::core::Error;
use windows::Win32::Foundation::ERROR_INVALID_PARAMETER;

static VERSION: Lazy<String> = Lazy::new(|| {
    format!("{}.{}", env!("CARGO_PKG_VERSION"), env!("BUILD_HASH"))
});

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct JsonSourceArgs {

    /// Specifies the JSON file to read
    #[arg(short, long, value_name = "Json file path")]
    file: Option<String>,

    /// Specifies the JSON text to read
    #[arg(short, long, value_name = "Json text")]
    text: Option<String>,
}

#[derive(Debug, Parser)]
#[command(author, version = VERSION.as_str(), about, arg_required_else_help(true))]
struct ProjsonApp {

    #[command(flatten)]
    json_src: JsonSourceArgs,

    /// Specifies the virtualization root directory path
    #[arg(short, long, value_name = "Virtual root directory path", required = true)]
    path: String,
}

impl ProjsonApp {
    fn parse_json_object(&self) -> windows::core::Result<Value> {
        if let Some(file) = &self.json_src.file {
            let mut file = match File::open(file.as_str()) {
                Ok(f) => f,
                Err(_) => return Err(Error::from(ERROR_INVALID_PARAMETER)),
            };

            let mut file_ctn = String::new();
            if let Err(_) = file.read_to_string(&mut file_ctn) {
                return Err(Error::from(ERROR_INVALID_PARAMETER));
            }

            match serde_json::from_str(&file_ctn) {
                Ok(val) => Ok(val),
                Err(_) => Err(Error::from(ERROR_INVALID_PARAMETER))
            }
        } else if let Some(text) = &self.json_src.text {
            match serde_json::from_str(&text) {
                Ok(val) => Ok(val),
                Err(_) => Err(Error::from(ERROR_INVALID_PARAMETER))
            }
        } else {
            Err(Error::from(ERROR_INVALID_PARAMETER))
        }
    }
}

fn main() -> windows::core::Result<()> {
    let app = ProjsonApp::parse();

    if let Err(err) = app.parse_json_object() {
        return Err(err);
    }

    Ok(())
}