use crate::models::*;
use console::style;
use emojis::*;
use futures::prelude::*;
use reqwest::{header, Client};
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;
use tokio::prelude::*;

mod emojis;
mod models;

#[allow(clippy::filter_map)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let cli = get_cli().await?;

    if let (Some(token), Some(file_id), Some(document_id)) =
        (cli.personal_access_token, cli.file_id, cli.document_id)
    {
        let scales = cli.file_scales;
        let formats = cli.file_extensions;
        let download_path = std::env::current_dir().unwrap().join(&cli.path);

        let client = get_client(&token)?;
        let frames = get_frames(&file_id, &document_id, &client).await?;
        let images = get_images(&frames, &client, &file_id, &scales, &formats).await;

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
            future::join_all(
                scales
                    .iter()
                    .filter(|&s| *s > 1)
                    .map(|s| tokio::fs::create_dir_all(download_path.join(format!("{}.0x", s)))),
            )
            .await;
            download_images(images, &client, &download_path).await?;
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

async fn get_cli() -> Result<Cli, Box<dyn Error>> {
    let cli: Cli = Cli::from_args();
    if cli.personal_access_token.is_some() {
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
) -> Result<Option<Frames>, Box<dyn Error>> {
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

async fn get_images(
    frames: &Option<Frames>,
    client: &Client,
    file_id: &str,
    scales: &[usize],
    formats: &[String],
) -> Vec<Image> {
    println!("{}  {}", LINK, style("Getting URLs from...").bold().green());
    if let Some(frames) = frames {
        let image_ids = frames
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let mut futures = vec![];
        for scale in scales {
            for format in formats {
                let fim = get_images_url_collection(&image_ids, client, file_id, *scale, format)
                    .map_ok(move |urls| to_images(frames, &urls, *scale, format));
                futures.push(fim);
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
                    f.name.clone(),
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
) -> Result<(), Box<dyn Error>> {
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
        let mut file =
            tokio::fs::File::create(&path.join(format!("{}.{}", i.name, i.format))).await?;
        file.write_all(&bytes).await?;
        Ok::<(), Box<dyn Error>>(())
    });

    future::join_all(futures).await;
    Ok(())
}
