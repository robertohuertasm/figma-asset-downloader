#![allow(clippy::non_ascii_literal)]
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;

type ImageId = String;
type ImageUrl = String;
pub type Frames = Vec<Node>;

#[derive(StructOpt, PartialEq, Debug, Deserialize)]
#[structopt(
    name("🌇  Figma Asset Downloader"),
    author("💻  Roberto Huertas <roberto.huertas@outlook.com>"),
    long_about("🧰   Small utility to help you download Figma assets directly to your computer.\n🦀  Humbly written with Rust. 🧡 \n🔗  https://github.com/robertohuertasm/figma-asset-downloader")
)]
pub struct Cli {
    /// Figma personal access token
    #[structopt(short = "t", long, requires_all(&["file-id", "document-id"]))]
    pub personal_access_token: Option<String>,
    /// File id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
    #[structopt(short, long, requires_all(&["personal-access-token", "document-id"]))]
    pub file_id: Option<String>,
    /// Document id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
    #[structopt(short, long, requires_all(&["personal-access-token", "file-id"]))]
    pub document_id: Option<String>,
    /// Path where assets will be downloaded
    #[structopt(short, long, default_value = "downloads")]
    #[serde(default = "default_path")]
    pub path: String,
    /// Extensions to export to in case there's no extension in the name of the asset: "png", "svg", "jpg", default: png
    #[structopt(short = "e", long, default_value = "png")]
    #[serde(default = "default_format")]
    pub file_extensions: Vec<String>,
    /// If true, file extensions will prevail over naming convention (asset_name.jpg)
    #[structopt(short = "r", long)]
    #[serde(default = "default_force_file_extensions")]
    pub force_file_extensions: bool,
    /// Scales to export to: 1, 2, 3, 4, default: 1
    #[structopt(short = "s", long, default_value = "1")]
    #[serde(default = "default_scale")]
    pub file_scales: Vec<usize>,
    /// Name of the figma-asset-downloader configuration
    #[structopt(short = "c", long, default_value = "fad.toml")]
    #[serde(default)]
    pub config_path: String,
    /// Optimizes png images. You can set a level from 1 to 6. 2 to 4 recommended.
    #[structopt(long)]
    pub opt_png_level: Option<u8>,
    /// Optimizes jpg images. You can set a level from 1 to 100. 80 recommended.
    #[structopt(long)]
    pub opt_jpg_level: Option<u8>,
    #[structopt(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(StructOpt, Debug, PartialEq, Deserialize)]
pub enum SubCommands {
    #[structopt(about = "Validates the result of the import with a manifest (fad_manifest.toml)")]
    ValidateManifest {
        #[structopt(short = "p", long, default_value = "fad_manifest.toml")]
        #[serde(default)]
        path: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    /// Files that must be imported. Don't use extension in case you want to use [file_extensions] arguments.
    pub files: Vec<String>,
    /// Extensions to export to in case there's no extension in the name of the asset: "png", "svg", "jpg", default: png
    #[serde(default = "default_format")]
    pub file_extensions: Vec<String>,
    /// Scales to export to: 1, 2, 3, 4, default: 1
    #[serde(default = "default_scale")]
    pub file_scales: Vec<usize>,
    /// Path where assets will be downloaded
    #[serde(default = "default_path")]
    pub path: String,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            files: vec![],
            file_extensions: default_format(),
            file_scales: default_scale(),
            path: default_path(),
        }
    }
}

impl Manifest {
    #[cfg(test)]
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }
}

// methods below have been implemented for default values when using fad.toml
fn default_scale() -> Vec<usize> {
    vec![1]
}

fn default_format() -> Vec<String> {
    vec!["png".to_string()]
}

fn default_path() -> String {
    "downloads".to_string()
}

const fn default_force_file_extensions() -> bool {
    false
}
// end of default values for serde

#[derive(Debug, Deserialize, Clone)]
pub struct Page {
    pub name: String,
    pub nodes: HashMap<String, Document>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Document {
    pub document: Node,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum NodeType {
    DOCUMENT,
    CANVAS,
    FRAME,
    GROUP,
    VECTOR,
    BOOLEAN_OPERATION,
    STAR,
    LINE,
    ELLIPSE,
    REGULAR_POLYGON,
    RECTANGLE,
    TEXT,
    SLICE,
    COMPONENT,
    INSTANCE,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageUrlCollection {
    pub images: HashMap<ImageId, ImageUrl>,
}

pub struct Image {
    pub id: String,
    pub name: String,
    pub scale: usize,
    pub format: String,
    pub url: String,
}

impl Image {
    pub fn new(id: String, name: &str, scale: usize, format: String, url: String) -> Self {
        Self {
            id,
            name: remove_extension(name),
            scale,
            format,
            url,
        }
    }
}

fn remove_extension(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .expect("Some unexpected error happened removing the extension of an image")
        .to_string()
}
