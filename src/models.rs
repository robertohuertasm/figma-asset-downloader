#![allow(clippy::non_ascii_literal)]
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;

type ImageId = String;
type ImageUrl = String;
pub type Frames = Vec<Node>;

const DEFAULT_PATH: &str = "downloads";
const DEFAULT_FILE_EXT: &str = "png";
const DEFAULT_FILE_SCALE: &str = "1";
const DEFAULT_CONFIG_PATH: &str = "fad.toml";

#[derive(StructOpt, PartialEq, Debug, Deserialize)]
#[structopt(
    name("🌇  Figma Asset Downloader"),
    author("💻  Roberto Huertas <roberto.huertas@outlook.com>"),
    long_about("🧰   Small utility to help you download Figma assets directly to your computer.\n🦀  Humbly written with Rust. 🧡 \n🔗  https://github.com/robertohuertasm/figma-asset-downloader")
)]
pub struct Cli {
    /// Figma personal access token
    #[structopt(short = "t", long)]
    pub personal_access_token: Option<String>,
    /// File id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
    #[structopt(short, long)]
    pub file_id: Option<String>,
    /// Document id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
    #[structopt(short, long)]
    pub document_id: Option<String>,
    /// Path where assets will be downloaded
    #[structopt(short, long, default_value = DEFAULT_PATH)]
    #[serde(default = "default_path")]
    pub path: String,
    /// Extensions to export to in case there's no extension in the name of the asset: "png", "svg", "jpg", default: png
    #[structopt(short = "e", long, default_value = DEFAULT_FILE_EXT)]
    #[serde(default = "default_format")]
    pub file_extensions: Vec<String>,
    /// If true, file extensions will prevail over naming convention (asset_name.jpg)
    #[structopt(short = "r", long)]
    #[serde(default = "default_force_file_extensions")]
    pub force_file_extensions: bool,
    /// Scales to export to: 1, 2, 3, 4, default: 1
    #[structopt(short = "s", long, default_value = DEFAULT_FILE_SCALE)]
    #[serde(default = "default_scale")]
    pub file_scales: Vec<usize>,
    /// Name of the figma-asset-downloader configuration
    #[structopt(short = "c", long, default_value = DEFAULT_CONFIG_PATH)]
    #[serde(default)]
    pub config_path: String,
    /// Optimizes png images. You can set a level from 1 to 6. 2 to 4 recommended.
    #[structopt(long)]
    pub opt_png_level: Option<u8>,
    /// Optimizes jpg images. You can set a level from 1 to 100. 80 recommended.
    #[structopt(long)]
    pub opt_jpg_level: Option<u8>,
    /// If true, only new added images will be optimized. It's useful to only apply optimization to recently imported images and not to all of them.
    #[structopt(short = "v", long)]
    #[serde(default = "default_opt_only_on_validation")]
    pub opt_only_on_validation: bool,
    #[structopt(subcommand)]
    pub subcommands: Option<SubCommands>,
}

impl Cli {
    /// Adds non default values from another cli
    pub fn add_non_defaults(&mut self, other_cli: Self) {
        self.subcommands = other_cli.subcommands;
        if other_cli.opt_only_on_validation {
            self.opt_only_on_validation = true;
        }
        if other_cli.force_file_extensions {
            self.force_file_extensions = true;
        }
        if other_cli.personal_access_token.is_some() {
            self.personal_access_token = other_cli.personal_access_token;
        }
        if other_cli.file_id.is_some() {
            self.file_id = other_cli.file_id;
        }
        if other_cli.document_id.is_some() {
            self.document_id = other_cli.document_id;
        }
        if other_cli.opt_png_level.is_some() {
            self.opt_png_level = other_cli.opt_png_level;
        }
        if other_cli.opt_jpg_level.is_some() {
            self.opt_jpg_level = other_cli.opt_jpg_level;
        }
        if other_cli.path != default_path() {
            self.path = other_cli.path;
        }
        if other_cli.config_path != *DEFAULT_CONFIG_PATH {
            self.config_path = other_cli.config_path;
        }
        if other_cli
            .file_extensions
            .iter()
            .filter(|&x| x != DEFAULT_FILE_EXT)
            .count()
            > 0
        {
            self.file_extensions = other_cli.file_extensions;
        }
        if other_cli
            .file_scales
            .iter()
            .filter(|&&x| x != DEFAULT_FILE_SCALE.parse::<usize>().unwrap())
            .count()
            > 0
        {
            self.file_scales = other_cli.file_scales;
        }
    }
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
    vec![DEFAULT_FILE_EXT.to_string()]
}

fn default_path() -> String {
    DEFAULT_PATH.to_string()
}

const fn default_force_file_extensions() -> bool {
    false
}

const fn default_opt_only_on_validation() -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_default_cli() -> Cli {
        Cli {
            personal_access_token: None,
            file_id: None,
            document_id: None,
            path: "".to_string(),
            file_scales: vec![1],
            file_extensions: vec![DEFAULT_FILE_EXT.to_owned()],
            force_file_extensions: false,
            config_path: "".to_string(),
            opt_png_level: None,
            opt_jpg_level: None,
            opt_only_on_validation: false,
            subcommands: None,
        }
    }

    #[test]
    fn cli_add_non_defaults_add_subcommands() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.subcommands = Some(SubCommands::ValidateManifest {
            path: "a".to_string(),
        });

        cli.add_non_defaults(other);

        assert!(cli.subcommands.is_some());
        assert_eq!(
            cli.subcommands,
            Some(SubCommands::ValidateManifest {
                path: "a".to_string(),
            })
        );
    }

    #[test]
    fn cli_add_non_defaults_add_opt_only_on_validation_if_true() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        assert!(!cli.opt_only_on_validation);

        other.opt_only_on_validation = true;
        cli.add_non_defaults(other);

        assert!(cli.opt_only_on_validation);
    }

    #[test]
    fn cli_add_non_defaults_add_force_file_extensions_if_true() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        assert!(!cli.force_file_extensions);

        other.force_file_extensions = true;
        cli.add_non_defaults(other);

        assert!(cli.force_file_extensions);
    }

    #[test]
    fn cli_add_non_defaults_add_pat_if_some() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.personal_access_token = Some("x".to_string());
        cli.add_non_defaults(other);

        assert!(cli.personal_access_token.is_some());
        assert_eq!(cli.personal_access_token, Some("x".to_string()));
    }

    #[test]
    fn cli_add_non_defaults_add_file_id_if_some() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.file_id = Some("x".to_string());
        cli.add_non_defaults(other);

        assert!(cli.file_id.is_some());
        assert_eq!(cli.file_id, Some("x".to_string()));
    }

    #[test]
    fn cli_add_non_defaults_add_document_id_if_some() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.document_id = Some("x".to_string());
        cli.add_non_defaults(other);

        assert!(cli.document_id.is_some());
        assert_eq!(cli.document_id, Some("x".to_string()));
    }

    #[test]
    fn cli_add_non_defaults_add_opt_png_level_if_some() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.opt_png_level = Some(10);
        cli.add_non_defaults(other);

        assert!(cli.opt_png_level.is_some());
        assert_eq!(cli.opt_png_level, Some(10));
    }

    #[test]
    fn cli_add_non_defaults_add_opt_jpg_level_if_some() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.opt_jpg_level = Some(10);
        cli.add_non_defaults(other);

        assert!(cli.opt_jpg_level.is_some());
        assert_eq!(cli.opt_jpg_level, Some(10));
    }

    #[test]
    fn cli_add_non_defaults_add_path_if_not_default() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        assert!(cli.path.is_empty());
        other.path = "x".to_string();
        cli.add_non_defaults(other);

        assert_eq!(cli.path, "x");
    }

    #[test]
    fn cli_add_non_defaults_add_config_path_if_not_default() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        assert!(cli.config_path.is_empty());
        other.config_path = "x".to_string();
        cli.add_non_defaults(other);

        assert_eq!(cli.config_path, "x");
    }

    #[test]
    fn cli_add_non_defaults_add_file_extensions_if_not_default() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.file_extensions = vec!["x".to_string()];
        cli.add_non_defaults(other);

        assert_eq!(cli.file_extensions, vec!["x".to_string()]);
    }

    #[test]
    fn cli_add_non_defaults_add_file_scales_if_not_default() {
        let mut cli = build_default_cli();
        let mut other = build_default_cli();

        other.file_scales = vec![1, 2, 3];
        cli.add_non_defaults(other);

        assert_eq!(cli.file_scales, vec![1, 2, 3]);
    }
}
