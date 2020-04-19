## Intro

[![ActionsStatus](https://github.com/robertohuertasm/figma-asset-downloader/workflows/Build/badge.svg)](https://github.com/robertohuertasm/figma-asset-downloader/actions) [![Crates.io](https://img.shields.io/crates/v/figma-asset-downloader.svg)](https://crates.io/crates/figma-asset-downloader)

Small utility to help you download [Figma](https://www.figma.com/) assets directly to your computer.

## Motivation

It may be very useful to keep the assets of your app/web up to date with the latest designs that your Design Team produces in [Figma](https://www.figma.com/).

Let's say your Design Team puts all the `assets` in a specific page of a [Figma](https://www.figma.com/) document. This utility would **automatically fetch all the images** from there and keep your assets folder up to date with the latest resources. Nice, isn't it?

I built this tool while working on the [Evermind](https://evermind.health/) app at [Telefónica Alpha](https://www.alpha.company/). I realized there were no tools that could make it easy to keep in sync the app assets with the [Figma](https://www.figma.com/) assets, and I was a little bit tired of the [Figma manual export system](https://help.figma.com/hc/en-us/articles/360040028114-Getting-Started-with-Exports).

The tool is **open source** and has a **MIT license** so feel free to use it, modify it or extend it in any form if you need it.

I hope it can be useful for your team, too! :wink:

## Installation

You can currently get artifacts for `Windows`, `MacOS` & `Ubuntu`. 

Just download your preferred release from [GitHub releases](https://github.com/robertohuertasm/figma-asset-downloader/releases), unzip it and add the `fad` executable to your path.

You can compile it yourself if you need to use it in another `OS`:

```sh
cargo install figma-asset-downloader
```

## Requirements

You'll need to get a [Figma Personal Access Token](https://www.figma.com/developers/api#access-tokens) in order to use the tool. Follow [this link](https://www.figma.com/developers/api#access-tokens) to learn how to get it.

![Personal Access Token](./img/personal_access_token.png "Personal Access Token")

## Usage

The usage is fairly simple:

```sh
fad -t <personal-access-token> -f <file-id> -d <document-id>
```

To get the `file-id` and the `document-id`, you have to take a look at the `url` of the Figma page that you want to download the images from.

You should be seeing something similar to this:

`https://www.figma.com/file/FILE_ID/file_title?node-id=DOCUMENT_ID`

Just get that pieces of information and use them from the command line.

Bear in mind that the `document-id` may be url-encoded. Don't use it like that. Substitute `%3A` for `:`.

```sh
...node-id=323%3A471
# should be 321:471
```

**IMPORTANT**: Be sure to use the `node-id` of the page. Don't select any object in your Figma document or this `node-id` will not correspond to the page but to some of the elements inside it.

## Defaults

By default, all the images will be downloaded at `scale 1` and `png` format inside a folder called `downloads`.

If you want to change this, you can use any of the other options that this `cli` provides. Specifically, `-s` will accept a collection of scales (1,2,3,4...) and `-e` will allow you to define a collection of exporting format (`png`, `svg`, `pdf`, `jpeg`).

```sh
fad [-t personal-access-token] [-f file-id] [-d document-id] [-p download-folder-name] [-s 1 2 3 4] [-e png svg jpeg pdf] [-c configuration-file]
```

## Configuration file

If you don't want to manually provide the arguments every time you use the `cli` you can also use a configuration file.

By default, if you don't pass any parameter and just call `fad`, the tool will look for a configuration file called `fad.toml` in the root folder where you're executing the tool. If it can't find it, it will error.

This is an example of a `fad.toml` file:

```toml
personal_access_token = "30277-2c47420f-8d6b-4c6c-b170-2727b8999653"
file_id = "h92QKQ8iOkFlq0q6mA4UhX"
document_id =  "323:471"
path = "downloads"
file_extensions = ["png"]
file_scales = [1,2,3,4]
```

You can use another name but then you'd have to specify it every time you call `fad` like this:

```sh
fad -c new-fad-config-file-name.toml
```

## Image format

Regarding `image format`, we previously said that `png` is the default format unless you specify otherwise by using the `--file-extensions` or `-e` argument.

But there is another mechanism for you to decide, explicitly from [Figma](https://www.figma.com/), which will be the format of your asset.

For instance, if you want some asset to be downloaded only in `jpeg` format regardless of what you choose with the `--file-extensions` or `-e` argument, you must add a suffix to your asset in Figma containing the extension: `your_asset_name.jpeg`.

If you happen to be following this convention while creating your Figma documents, but you still want to download the images in a specific format then you must use the `--force-file-extensions` flag.

If you need more help just execute `fad -h`.

## Image optimization

[Figma](https://www.figma.com/) export API does not optimize the images. That's why this tool has also the ability to optimize `jpeg` and `png` formats. 

You just have to use the `--opt-jpg-level` and `--opt-png-level` options.

Bear in mind that if you choose to optimize the images, the process will take a little bit more time than usual.

## Options

```txt
USAGE:
    fad [FLAGS] [OPTIONS]

FLAGS:
    -r, --force-file-extensions    If true, file extensions will prevail over naming convention (asset_name.jpg)
    -h, --help                     Prints help information
    -V, --version                  Prints version information

OPTIONS:
    -c, --config-path <config-path>
            Name of the figma-asset-downloader configuration [default: fad.toml]

    -d, --document-id <document-id>
            Document id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)

    -e, --file-extensions <file-extensions>...
            Extensions to export to in case there's no extension in the name of the asset: "png", "svg", "jpg", default:
            png [default: png]
    -f, --file-id <file-id>
            File id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)

    -s, --file-scales <file-scales>...                     Scales to export to: 1, 2, 3, 4 [default: 1]
        --opt-jpg-level <opt-jpg-level>
            Optimizes jpg images. You can set a level from 1 to 100. 80 recommended

        --opt-png-level <opt-png-level>
            Optimizes png images. You can set a level from 1 to 6. 2 to 4 recommended

    -p, --path <path>                                      Path where assets will be downloaded [default: downloads]
    -t, --personal-access-token <personal-access-token>    Figma personal access token
```

**NOTE**: If you provide any arguments to the `cli`, they will take precedence over the `configuration` file. `-t, -f, -d` are always mandatory if at least any one of them is manually provided.
