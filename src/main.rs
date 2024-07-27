use std::borrow::Cow;
use std::fs::File;
use std::{fs, io};
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use clap::{Args, Parser};
use once_cell::sync::Lazy;
use serde_json::{Value};
use windows_projfs::{DirectoryEntry, DirectoryInfo, FileInfo, ProjectedFileSystem, ProjectedFileSystemSource};

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
    path: PathBuf,
}

impl ProjsonApp {
    fn parse_json_object(&self) -> anyhow::Result<Value> {
        let json_text: Cow<str> = match &self.json_src.text {
            None => {
                if let Some(file) = &self.json_src.file {
                    let mut file = File::open(file.as_str())?;
                    let mut file_ctn = String::new();
                    file.read_to_string(&mut file_ctn)?;
                    Cow::Owned(file_ctn)
                } else {
                    unreachable!()
                }
            }
            Some(text) => Cow::Borrowed(text)
        };
        let json_text = json_text.trim();
        Ok(serde_json::from_str(&json_text)?)
    }

    fn start_and_wait(self) -> anyhow::Result<()> {
        if self.path.exists() {
            log::error!("Target {} already exists", &self.path.display());
            return Ok(());
        } else {
            fs::create_dir_all(&self.path)?;
        };

        let val = self.parse_json_object()?;
        log::info!("Virtual path: {}", self.path.display());

        log::info!("Press any key to continue...");
        let _prj = ProjectedFileSystem::new(&self.path, JsonProvider{ val })?;
        let _ = io::stdin().read(&mut [0u8]).unwrap();

        log::info!("Stopped");
        fs::remove_dir_all(&self.path)?;
        Ok(())
    }
}

pub struct JsonProvider {
    val: Value,
}

fn get_path<'a>(value: &'a Value, path: &'a Path) -> Option<&'a Value> {
    let dir_part: Vec<&str> = path.components()
        .filter_map(|c| match c {
            Component::Prefix(p) => p.as_os_str().to_str(),
            Component::RootDir => Some("\\"),
            Component::CurDir | Component::ParentDir => Some("."),
            Component::Normal(s) => s.to_str(),
        })
        .collect();
    dir_part.iter().fold(Some(value), |acc, &key| {
        acc.and_then(|v| v.as_object().and_then(|m| m.get(key)))
    })
}

impl ProjectedFileSystemSource for JsonProvider {
    fn list_directory(
        &self,
        path: &Path
    ) -> Vec<windows_projfs::DirectoryEntry> {

        let curr_path_val = get_path(&self.val, path);
        if let None = curr_path_val {
            return Vec::new();
        }

        let obj = curr_path_val.unwrap().as_object();
        match obj {
            None => Vec::new(),
            Some(obj) => {
                let mut entries = Vec::new();
                for (key, val) in obj {
                    let entry = match val {
                        Value::Null => { None }
                        Value::Bool(v) => {
                            let ctn = if *v { "true".to_string() } else { "false".to_string() };
                            Some(DirectoryEntry::File(FileInfo {
                                file_name: key.clone(),
                                file_size: ctn.len() as u64,
                                ..Default::default()
                            }))
                        }
                        Value::Number(v) => {
                            Some(DirectoryEntry::File(FileInfo {
                                file_name: key.clone(),
                                file_size: v.to_string().len() as u64,
                                ..Default::default()
                            }))
                        }
                        Value::String(v) => {
                            Some(DirectoryEntry::File(FileInfo {
                                file_name: key.clone(),
                                file_size: v.len() as u64,
                                ..Default::default()
                            }))
                        }
                        Value::Array(_) | Value::Object(_) => {
                            Some(DirectoryEntry::Directory(DirectoryInfo {
                                directory_name: key.clone(),
                                ..Default::default()
                            }))
                        }
                    };
                    if let Some(entry) = entry {
                        entries.push(entry)
                    }
                }

                entries
            }
        }
    }

    fn stream_file_content(
        &self,
        path: &Path,
        byte_offset: usize,
        length: usize
    ) -> io::Result<Box<dyn Read>> {
        let curr_path_val = get_path(&self.val, path);
        if let None = curr_path_val {
            return Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
        }

        let ctn = match curr_path_val.unwrap() {
            Value::Null => {
                Some("".to_string())
            }
            Value::Bool(v) => {
                Some(if *v { "true".to_string() } else { "true".to_string() })
            }
            Value::Number(v) => {
                Some(v.to_string())
            }
            Value::String(v) => {
                Some(v.clone())
            }
            Value::Array(_) | Value::Object(_) => {
                None
            }
        };

        match ctn {
            None => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input")),
            Some(ctn) => {
                Ok(Box::new(Cursor::new(
                    ctn.as_bytes()[byte_offset..(byte_offset + length)].to_owned()
                )))
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // 设置日志级别为 Info
        .init();
    ProjsonApp::parse().start_and_wait()
}
