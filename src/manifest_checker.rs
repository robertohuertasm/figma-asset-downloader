#![allow(dead_code)]

use crate::{emojis, models::Manifest};
use async_trait::async_trait;
use console::style;
use scan_dir::ScanDir;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub enum ManifestError {
    Generic,
    Parse(String),
    IO(String),
}

impl Error for ManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::Generic => write!(f, "Generic Error"),
            Self::Parse(s) => write!(f, "Error trying to parse the manifest: {}", s),
            Self::IO(s) => write!(f, "Error trying to read a file/directory: {}", s),
        }
    }
}

impl From<toml::de::Error> for ManifestError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error.to_string())
    }
}

impl From<std::io::Error> for ManifestError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error.to_string())
    }
}

#[async_trait]
pub trait ManifestReader {
    async fn read_manifest(&self) -> Result<Manifest, ManifestError>;
    async fn read_assets(&self, assets_dir_path: &PathBuf) -> Result<Vec<String>, ManifestError>;
}

pub struct TokioManifestReader<'a> {
    manifest_path: &'a PathBuf,
}

#[async_trait]
impl ManifestReader for TokioManifestReader<'_> {
    async fn read_manifest(&self) -> Result<Manifest, ManifestError> {
        let manifest_str = tokio::fs::read_to_string(&self.manifest_path).await?;
        let manifest = toml::from_str(&manifest_str)?;
        Ok(manifest)
    }
    async fn read_assets(&self, assets_dir_path: &PathBuf) -> Result<Vec<String>, ManifestError> {
        let mut all_files: Vec<String> = vec![];
        let walk_result = ScanDir::files().walk(assets_dir_path, |wlkr| {
            for (entry, _) in wlkr {
                let entry_str = entry
                    .path()
                    .strip_prefix(assets_dir_path)
                    .expect("unable to remove prefix")
                    .display()
                    .to_string();

                all_files.push(entry_str);
            }
        });
        if let Err(errors) = walk_result {
            return Err(ManifestError::IO(format!(
                "Errors reading the assets directory: {:?}",
                errors
            )));
        }
        Ok(all_files)
    }
}

pub struct ManifestChecker<T: ManifestReader> {
    reader: T,
}

#[allow(clippy::use_self)]
impl<T: ManifestReader> ManifestChecker<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn with_tokio_reader<'a>(
        manifest_path: &'a PathBuf,
    ) -> ManifestChecker<TokioManifestReader<'a>> {
        ManifestChecker::new(TokioManifestReader { manifest_path })
    }

    pub async fn check(&self) -> Result<ManifestInfo, ManifestError> {
        let manifest = self.reader.read_manifest().await?;
        let assets_dir_path = std::env::current_dir()?.join(&manifest.path);
        let assets = self.reader.read_assets(&assets_dir_path).await?;
        Ok(Self::compare_results(manifest, assets))
    }

    fn compare_results(manifest: Manifest, assets: Vec<String>) -> ManifestInfo {
        let mut new_assets = vec![];
        let extensions = &manifest.file_extensions;
        let scales = &manifest.file_scales;

        let mut manifest_map = manifest
            .files
            .into_iter()
            .fold(HashMap::new(), |mut acc, file| {
                let mut temp_files = vec![];

                // set the extensions if needed
                if Path::new(&file).extension().is_none() {
                    for ext in extensions {
                        temp_files.push(format!("{}.{}", file, ext));
                    }
                } else {
                    temp_files.push(file);
                }

                // set the folder if needed
                let mut temp_files_scales = vec![];
                for scale in scales.iter().filter(|&s| s > &1) {
                    for tf in &temp_files {
                        temp_files_scales.push(format!("{}.0x/{}", scale, tf));
                    }
                }

                temp_files.append(&mut temp_files_scales);

                for temp_file in temp_files {
                    acc.insert(temp_file, false);
                }
                acc
            });

        for asset in assets {
            if let Some(x) = manifest_map.get_mut(&asset) {
                *x = true;
            } else {
                new_assets.push(asset.clone());
            }
        }

        let mut missing_assets = manifest_map
            .into_iter()
            .filter_map(|(key, val)| if val { None } else { Some(key) })
            .collect::<Vec<_>>();

        missing_assets.sort();
        new_assets.sort();

        ManifestInfo::default()
            .with_new_assets(new_assets)
            .with_missing_assets(missing_assets)
    }
}

#[derive(Default, Debug)]
pub struct ManifestInfo {
    pub new_assets: Option<Vec<String>>,
    pub missing_assets: Option<Vec<String>>,
}

impl ManifestInfo {
    /// Adds new assets collection
    pub fn with_new_assets(mut self, assets: Vec<String>) -> Self {
        if !assets.is_empty() {
            self.new_assets = Some(assets);
        }
        self
    }

    /// Adds missing assets collection
    pub fn with_missing_assets(mut self, assets: Vec<String>) -> Self {
        if !assets.is_empty() {
            self.missing_assets = Some(assets);
        }
        self
    }

    /// Prints the information about the manifest
    pub fn print_info(&self) {
        if let Some(missing_assets) = &self.missing_assets {
            println!(
                "{} {}",
                emojis::ERROR,
                style("There are some assets missing").red().bold()
            );
            for asset in missing_assets {
                println!("    {}", style(asset).red());
            }
        }
        if let Some(new_assets) = &self.new_assets {
            println!(
                "{} {}",
                emojis::PLANT,
                style("There are some new assets").green().bold()
            );
            for asset in new_assets {
                println!("    {}", style(asset).green());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    struct MockReader {
        pub read_manifest_result: Manifest,
        pub read_assets_result: Vec<String>,
    }

    impl MockReader {
        fn new(manifest_result: Manifest, assets_result: Vec<String>) -> Self {
            Self {
                read_manifest_result: manifest_result,
                read_assets_result: assets_result,
            }
        }
    }

    #[async_trait]
    impl ManifestReader for MockReader {
        async fn read_manifest(&self) -> Result<Manifest, ManifestError> {
            Ok(self.read_manifest_result.clone())
        }

        async fn read_assets(&self, _: &PathBuf) -> Result<Vec<String>, ManifestError> {
            Ok(self.read_assets_result.clone())
        }
    }

    fn asset_builder(vec: Vec<&str>) -> Vec<String> {
        vec.into_iter().map(String::from).collect()
    }

    #[tokio::test]
    async fn manifest_should_have_all_none_if_assets_and_manifest_are_ok() -> anyhow::Result<()> {
        let assets = asset_builder(vec!["a.jpg", "b.jpg"]);
        let manifest = Manifest::default().with_files(assets.clone());
        let mock_reader = MockReader::new(manifest, assets);
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_none());
        assert!(result.missing_assets.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn manifest_should_have_new_assets_if_new_assets_are_added() -> anyhow::Result<()> {
        let manifest = Manifest::default().with_files(asset_builder(vec!["a.jpg", "b.jpg"]));
        let assets = asset_builder(vec!["a.jpg", "b.jpg", "c.jpg"]);
        let mock_reader = MockReader::new(manifest, assets.clone());
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_some());
        assert_eq!(result.new_assets.unwrap()[0], assets[2]);
        assert!(result.missing_assets.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn manifest_should_have_missing_assets_if_assets_are_missing() -> anyhow::Result<()> {
        let manifest_files = asset_builder(vec!["a.jpg", "b.jpg"]);
        let manifest = Manifest::default().with_files(manifest_files.clone());
        let assets = asset_builder(vec!["a.jpg"]);
        let mock_reader = MockReader::new(manifest, assets);
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_none());
        assert!(result.missing_assets.is_some());
        assert_eq!(result.missing_assets.unwrap()[0], manifest_files[1]);
        Ok(())
    }

    #[tokio::test]
    async fn manifest_should_use_extensions_when_files_have_no_extension() -> anyhow::Result<()> {
        let manifest = Manifest {
            file_extensions: vec!["png".to_string(), "svg".to_string()],
            file_scales: vec![1],
            files: asset_builder(vec!["a", "b"]),
            path: "".to_owned(),
        };
        let assets = asset_builder(vec!["a.png", "a.svg", "b.png", "b.svg"]);
        let mock_reader = MockReader::new(manifest, assets);
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_none());
        assert!(result.missing_assets.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn manifest_should_not_use_extensions_when_files_have_extensions() -> anyhow::Result<()> {
        let manifest = Manifest {
            file_extensions: vec!["png".to_string(), "svg".to_string()],
            file_scales: vec![1],
            files: asset_builder(vec!["a", "b", "c.svg", "d.jpg"]),
            path: "".to_owned(),
        };
        let assets = asset_builder(vec!["a.png", "a.svg", "b.png", "b.svg", "c.svg", "d.jpg"]);
        let mock_reader = MockReader::new(manifest, assets);
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_none());
        assert!(result.missing_assets.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn manifest_should_use_scales() -> anyhow::Result<()> {
        let manifest = Manifest {
            file_extensions: vec!["png".to_string(), "svg".to_string()],
            file_scales: vec![1, 2, 3],
            files: asset_builder(vec!["a"]),
            path: "".to_owned(),
        };
        let assets = asset_builder(vec![
            "a.png",
            "a.svg",
            "2.0x/a.png",
            "2.0x/a.svg",
            "3.0x/a.png",
            "3.0x/a.svg",
        ]);
        let mock_reader = MockReader::new(manifest, assets);
        let checker = ManifestChecker::new(mock_reader);
        let result = checker.check().await?;

        assert!(result.new_assets.is_none());
        assert!(result.missing_assets.is_none());
        Ok(())
    }
}
