# Figma Asset Downloader

[![ActionsStatus](https://github.com/robertohuertasm/figma-asset-downloader/workflows/Build/badge.svg)](https://github.com/robertohuertasm/figma-asset-downloader/actions) [![Crates.io](https://img.shields.io/crates/v/figma-asset-downloader.svg)](https://crates.io/crates/figma-asset-downloader)

Small utility to help you download [Figma](https://www.figma.com/) assets directly to your computer

![cli demo](./img/demo.gif "cli demo")

## Motivation

It may be very useful to keep the assets of your app/web up to date. Let's say your Design Team puts all the `assets` in a specific page of a [Figma](https://www.figma.com/) document. This utility would automatically fetch all the images from there and keep your assets folder up to date with the latest resources. Nice, isn't it?

## Installation

You can compile it yourself:

```sh
cargo install figma-asset-downloader
```

Or you can download an OS specific executable from [GitHub releases](https://github.com/robertohuertasm/figma-asset-downloader/releases) and add it to your path.

Currently, you can get artifacts for `Windows`, `MacOS` & `Ubuntu`.

Remember that if you need to use it in another `OS` you can compile it yourself.

## Requirements

You'll need to get a [Figma Personal Access Token](https://www.figma.com/developers/api#access-tokens).

![Personal Access Token](https://github.com/robertohuertasm/figma-asset-downloader/raw/master/img/personal_access_token.png "Personal Access Token")

## Usage

The usage is fairly simple:

```sh
fad -t <personal-access-token> -f <file-id> -d <document-id>
```

To get the `file-id` and the `document-id`, you have to take a look at the `url` of the Figma page that you want to download the images from.

You should be seeing something similar to this:

`https://www.figma.com/file/FILE_ID/file_title?node-id=DOCUMENT_ID`

Just get that pieces of information and use them from the command line. Beware that the `document-id` may be url-encoded. Don't use it like that. Substitute `%3A` for `:`.

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

If you need more help just execute `fad -h`.

## Configuration file

If you don't want to manually provide the arguments all the time you can also use a configuration file.

By default, if you don't pass any parameter and just call `fad`, it will look for a configuration file called `fad.toml` in the folder where you're using the tool.

This is an example of a `fad.toml` file:

```toml
personal_access_token = "30277-2c47420f-8d6b-4c6c-b170-2727b8999653"
file_id = "h92QKQ8iOkFlq0q6mA4UhX"
document_id =  "323:471"
path = "downloads"
file_extensions = ["png"]
file_scales = [1,2,3,4]
```

You can use another name but then you'll have to specify it every time you call `fad` like this:

```sh
fad -c new-fad-config-file-name.toml
```

## Options

* `-t`: Figma Personal Access Token
* `-f`: File id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
* `-d`: Document id (www.figma.com/file/FILE_ID/title?node-id=DOCUMENT_ID)
* `-p`: Path where assets will be downloaded
* `-e`: Extensions to export to: "png", "svg", "jpeg", default: png
* `-s`: Scales to export to: 1, 2, 3, 4
* `-c`: Name of the figma-asset-downloader configuration

**NOTE**: If you provide any arguments to the `cli`, they will take precedence. `-t, -f, -d` are always mandatory if at least any one of them is manually provided.
