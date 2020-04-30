#![allow(clippy::wildcard_imports)]
use crate::models::*;
use console::style;
use emojis::*;
use futures::prelude::*;
use manifest_checker::{ManifestChecker, TokioManifestReader};
use reqwest::{header, Client};
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;
use tokio::prelude::*;

mod emojis;
mod manifest_checker;
mod models;

#[allow(clippy::filter_map)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let cli = get_cli().await?;

    let download_path: PathBuf = std::env::current_dir()?.join(&cli.path);

    if cli.subcommands.is_some() {
        // we have only one subcommand so no need to get more details than this
        let manifest_path = PathBuf::from("fad_manifest.toml");
        let manifest_checker =
            ManifestChecker::<TokioManifestReader>::with_tokio_reader(&manifest_path);
        let manifest_result = manifest_checker.check().await;
        match manifest_result {
            Ok(manifest) => {
                manifest.print_info();
            }
            Err(e) => {
                println!(
                    "{} {}",
                    ERROR,
                    style(format!(
                        "Some error occurred while trying to work with the manifest:\n\n{}",
                        e
                    ))
                    .bold()
                    .red(),
                );
            }
        }
    } else if let (Some(token), Some(file_id), Some(document_id)) =
        (cli.personal_access_token, cli.file_id, cli.document_id)
    {
        let scales = cli.file_scales;
        let formats = cli.file_extensions;
        let force_extensions = cli.force_file_extensions;

        let client = get_client(&token)?;
        let frames = get_frames(&file_id, &document_id, &client).await?;
        let images = get_images(
            &frames,
            &client,
            &file_id,
            &scales,
            &formats,
            force_extensions,
        )
        .await;

        if images.is_empty() {
            println!(
                "{}  {}",
                INFO,
                style("No images found to download").bold().blue()
            );
        } else {
            println!(
                "{}  {}",
                FOLDER,
                style("Creating the folder structure...").bold().green()
            );
            future::join_all(scales.iter().enumerate().map(|(i, s)| {
                if i == 0 {
                    tokio::fs::create_dir_all(download_path.clone())
                } else {
                    tokio::fs::create_dir_all(download_path.join(format!("{}.0x", s)))
                }
            }))
            .await;
            download_images(
                images,
                &client,
                &download_path,
                cli.opt_png_level,
                cli.opt_jpg_level,
            )
            .await?;
        }
        println!(
            "{}  {} {}  {}  {}",
            THUMB,
            style("We're done!").bold().green(),
            HEART,
            GIFT,
            CRAB
        );
    } else {
        println!(
            "{}  {}",
            ERROR,
            style("Some arguments are missing. Check access token, file id or document id.")
                .bold()
                .red(),
        );
    }
    println!(
        "{}  {}",
        CLOCK,
        style(format!("It took {} secs.", start.elapsed().as_secs()))
            .bold()
            .blue(),
    );
    Ok(())
}

async fn get_cli() -> anyhow::Result<Cli> {
    let cli: Cli = Cli::from_args();
    if cli.personal_access_token.is_some() || cli.subcommands.is_some() {
        Ok(cli)
    } else {
        let config_str = tokio::fs::read_to_string(&cli.config_path)
            .await
            .map_err(|e| {
                println!(
                    "{}  {}",
                    ERROR,
                    style("The provided configuration file was not found!")
                        .bold()
                        .red(),
                );
                e
            })?;
        let cli: Cli = toml::from_str(&config_str).map_err(|e| {
            // NOTE: this should never happen while the cli options keep being all optional.
            // we keep it here just in case something changes in the future.
            println!(
                "{}  {}",
                ERROR,
                style("An error occurred trying to parse the config file.")
                    .bold()
                    .red(),
            );
            e
        })?;
        Ok(cli)
    }
}

fn get_client(token: &str) -> Result<Client, reqwest::Error> {
    println!("{}  {}", ROCKET, style("Preparing...").bold().green());
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-Figma-Token",
        header::HeaderValue::from_str(token).unwrap(),
    );
    reqwest::Client::builder().default_headers(headers).build()
}

async fn get_frames(
    file_id: &str,
    document_id: &str,
    client: &Client,
) -> anyhow::Result<Option<Frames>> {
    let url = format!(
        "https://api.figma.com/v1/files/{}/nodes?ids={}",
        file_id, document_id,
    );
    println!(
        "{}  {}\n{} {}",
        FRAME,
        style("Getting Frames from...").bold().green(),
        LINK,
        url,
    );
    let mut page: Page = client.get(&url).send().await?.json().await.map_err(|e| {
        let error_message = "Check the values of your configuration. Is the URL ok?";
        println!("{}  {}", ERROR, style(error_message).bold().red());
        e
    })?;
    let document_node = page.nodes.remove(document_id).map(|doc| doc.document);
    let frames = document_node
        .map(|doc| {
            doc.children.map(|nodes| {
                nodes
                    .into_iter()
                    .filter(|node| node.node_type == NodeType::FRAME)
                    .collect::<Frames>()
            })
        })
        .flatten();

    Ok(frames)
}

async fn get_images<'a>(
    frames: &Option<Frames>,
    client: &Client,
    file_id: &str,
    scales: &[usize],
    formats: &[String],
    force_extensions: bool,
) -> Vec<Image> {
    println!("{}  {}", LINK, style("Getting URLs from...").bold().green());
    if let Some(frames) = frames {
        let mut free_images = vec![];
        let mut png_images = vec![];
        let mut jpg_images = vec![];
        let mut svg_images = vec![];
        let mut pdf_images = vec![];

        for frame in frames {
            let id = frame.id.as_str();
            if force_extensions {
                free_images.push(id);
            } else {
                match Path::new(&frame.name)
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                {
                    Some("png") => png_images.push(id),
                    Some("jpeg") | Some("jpg") => jpg_images.push(id),
                    Some("pdf") => pdf_images.push(id),
                    Some("svg") => svg_images.push(id),
                    _ => free_images.push(id),
                }
            }
        }

        let free_image_ids = free_images.join(",");
        let png_image_ids = png_images.join(",");
        let jpg_image_ids = jpg_images.join(",");
        let svg_image_ids = svg_images.join(",");
        let pdf_image_ids = pdf_images.join(",");

        let mut futures = vec![];

        let mut add_future_images = |image_ids: &'a str, format, scale| {
            if !image_ids.is_empty() {
                futures.push(
                    get_images_url_collection(image_ids, client, file_id, scale, format)
                        .map_ok(move |urls| to_images(frames, &urls, scale, format)),
                );
            }
        };

        for scale in scales {
            add_future_images(&png_image_ids, "png", *scale);
            add_future_images(&jpg_image_ids, "jpg", *scale);
            add_future_images(&svg_image_ids, "svg", *scale);
            add_future_images(&pdf_image_ids, "pdf", *scale);

            for format in formats {
                add_future_images(&free_image_ids, format, *scale);
            }
        }

        future::join_all(futures)
            .await
            .into_iter()
            .filter_map(std::result::Result::ok)
            .flatten()
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

async fn get_images_url_collection(
    image_ids: &str,
    client: &Client,
    file_id: &str,
    scale: usize,
    format: &str,
) -> Result<ImageUrlCollection, reqwest::Error> {
    let url = format!(
        "https://api.figma.com/v1/images/{}?ids={}&scale={}&format={}",
        file_id, image_ids, scale, format,
    );
    println!("{}  {}", LINK, url);
    client
        .get(&url)
        .send()
        .await?
        .json::<ImageUrlCollection>()
        .await
}

fn to_images(frames: &[Node], urls: &ImageUrlCollection, scale: usize, format: &str) -> Vec<Image> {
    frames
        .iter()
        .filter_map(|f| {
            urls.images.get(&f.id).map(|url| {
                Image::new(
                    f.id.clone(),
                    &f.name,
                    scale.to_owned(),
                    format.to_owned(),
                    url.to_owned(),
                )
            })
        })
        .collect()
}

async fn download_images(
    images: Vec<Image>,
    client: &Client,
    download_path: &PathBuf,
    opt_png_level: Option<u8>,
    opt_jpg_level: Option<u8>,
) -> anyhow::Result<()> {
    println!(
        "{}  {}",
        DOWN,
        style("Downloading images...").bold().green()
    );
    let futures = images.into_iter().map(move |i| async move {
        let bytes = client.get(&i.url).send().await?.bytes().await?;
        let path = if i.scale == 1 {
            download_path.to_owned()
        } else {
            download_path.join(format!("{}.0x", i.scale))
        };
        let final_path = path.join(format!("{}.{}", i.name.trim(), i.format));
        let mut file = tokio::fs::File::create(&final_path).await?;
        file.write_all(&bytes).await?;
        if let Err(e) = optimize_image(&final_path, &i.format, opt_png_level, opt_jpg_level) {
            println!(
                "{} Error optimizing image {:?} => {:?}",
                ERROR, final_path, e
            );
        }
        Ok::<(), anyhow::Error>(())
    });

    future::join_all(futures).await;
    Ok(())
}

fn optimize_image(
    path: &Path,
    extension: &str,
    opt_png_level: Option<u8>,
    opt_jpg_level: Option<u8>,
) -> anyhow::Result<()> {
    match extension {
        "jpg" => {
            if let Some(lvl) = opt_jpg_level {
                let img = image::open(path)?;
                let dim = image::image_dimensions(path)?;
                let mut fw = std::fs::File::create(path)?;
                let mut enc = image::jpeg::JPEGEncoder::new_with_quality(&mut fw, lvl);
                enc.encode(&img.to_bytes(), dim.0, dim.1, img.color())?;
            }
        }
        "png" => {
            if let Some(lvl) = opt_png_level {
                let inf = oxipng::InFile::from(path);
                let ouf = oxipng::OutFile::Path(Some(path.into()));
                let opts = oxipng::Options::from_preset(lvl);
                oxipng::optimize(&inf, &ouf, &opts)?;
            }
        }
        _ => (),
    }
    Ok(())
}
