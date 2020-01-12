#![allow(clippy::non_ascii_literal)]
use serde::Deserialize;
use std::collections::HashMap;
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
    /// Extensions to export to: "png", "svg", "jpeg", default: png
    #[structopt(short = "e", long, default_value = "png")]
    #[serde(default = "default_format")]
    pub file_extensions: Vec<String>,
    /// Scales to export to: 1, 2, 3, 4
    #[structopt(short = "s", long, default_value = "1")]
    #[serde(default = "default_scale")]
    pub file_scales: Vec<u8>,
    /// Name of the figma-asset-downloader configuration
    #[structopt(short = "c", long, default_value = "fad.toml")]
    #[serde(default)]
    pub config_path: String,
}

// methods below have been implemented for default values when using fad.toml
fn default_scale() -> Vec<u8> {
    vec![1]
}

fn default_format() -> Vec<String> {
    vec!["png".to_string()]
}

fn default_path() -> String {
    "downloads".to_string()
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
    pub scale: u8,
    pub format: String,
    pub url: String,
}

impl Image {
    pub const fn new(id: String, name: String, scale: u8, format: String, url: String) -> Self {
        Self {
            id,
            name,
            scale,
            format,
            url,
        }
    }
}
