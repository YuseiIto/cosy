//! A command-line interface example for the `cosy` parser.
//!
//! This example demonstrates how to integrate `cosy` into a CLI tool that
//! can fetch content from various sources (local files, URLs, or Scrapbox/Cosense pages)
//! and output the parsed Abstract Syntax Tree (AST).
//!
//! # Usage
//!
//! ```sh
//! # From a local file
//! cargo run --example cli -- local <path>
//!
//! # From a Scrapbox page
//! cargo run --example cli -- title <project>/<page> [api-key]
//! ```

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use cosy;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::COOKIE;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use urlencoding::encode as encode_url;

/// Command-line interface example for the `cosy` cosense syntax parser.
#[derive(Parser)]
struct Args {
    /// The source from which to read the input.
    #[command(subcommand)]
    source: InputSource,
    /// Path to the input file to be parsed.
    input: Option<String>,
}

/// Defines the various input sources supported by this CLI.
#[derive(Subcommand)]
enum InputSource {
    /// Fetches a Scrapbox/Cosense page by title.
    ///
    /// Please provide an API key if the project is private.
    /// Title format: `project/page`
    Title {
        /// The page title in `project/page` format.
        title: String,
        /// Optional `connect.sid` cookie value for private projects.
        key: Option<String>,
    },
    /// Fetches raw markup content from a URL.
    Url {
        /// The URL to fetch content from.
        url: String,
    },
    /// Fetches exported JSON content from a URL and extracts the text.
    JsonUrl {
        /// The URL to fetch JSON content from.
        url: String,
    },
    /// Reads raw markup from a local file.
    Local {
        /// The path to the local file.
        path: String,
    },
    /// Reads exported JSON from a local file and extracts the text.
    JsonLocal {
        /// The path to the local JSON file.
        path: String,
    },
}

/// Structure representing the relevant parts of the Scrapbox API JSON response.
#[derive(Deserialize)]
struct PageJson {
    /// The lines of text on the page.
    lines: Vec<LineJson>,
}

/// Structure representing a single line in the Scrapbox JSON.
#[derive(Deserialize)]
struct LineJson {
    /// The text content of the line.
    text: String,
}

/// The entry point of the CLI example.
///
/// It fetches content based on the provided subcommand, parses it using `cosy`,
/// and prints the resulting AST.
fn main() -> Result<()> {
    let args = Args::parse();

    let input = match args.source {
        InputSource::Title { title, key } => fetch_by_page_title(&title, &key)?,
        InputSource::Local { path } => read_plain_local(&path)?,
        InputSource::Url { url } => fetch_plain_by_url(&url, &None)?,
        InputSource::JsonLocal { path } => read_json_local(&path)?,
        InputSource::JsonUrl { url } => fetch_json_by_url(&url, &None)?,
    };

    let mut content = input.as_str();
    let result = cosy::parse(&mut content, &());

    match result {
        Ok(nodes) => println!("{:#?}", nodes),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

/// Reads the contents of a local file as a plain string.
fn read_plain_local(path: &str) -> Result<String> {
    let mut content = String::new();
    let mut f = File::open(path).context("File not found.")?;
    f.read_to_string(&mut content)
        .context("Something went wrong reading the file.")?;

    Ok(content)
}

/// Fetches the body of a URL as a plain string.
fn fetch_plain_by_url(url: &str, key: &Option<String>) -> Result<String> {
    let mut req = Client::new().get(url);
    if let Some(key) = key {
        req = req.header(COOKIE, format!("connect.sid={key};"))
    }

    let res = req.send().context("Failed to send request.")?;

    match res.status() {
        StatusCode::OK => (),
        StatusCode::UNAUTHORIZED => {
            bail!("Unauthorized to fetch content. Please provide your api key.")
        }
        StatusCode::NOT_FOUND => bail!("Content not found"),
        _ => bail!("Failed to fetch content."),
    };

    let plain = res.text()?;

    Ok(plain)
}

/// Fetches a Scrapbox page and returns its combined text content.
fn fetch_by_page_title(title: &str, key: &Option<String>) -> Result<String> {
    let split_title: Vec<&str> = title.split('/').collect();
    if !(split_title.len() == 2) || split_title[0].is_empty() || split_title[1].is_empty() {
        bail!("Invalid page title. It must be `project/page` style.")
    }

    let project_name = encode_url(split_title[0]);
    let page_title = encode_url(split_title[1]);

    let url = format!("https://scrapbox.io/api/pages/{project_name}/{page_title}");

    fetch_json_by_url(&url, key)
}

/// Reads a local JSON file and extracts the text lines.
fn read_json_local(path: &str) -> Result<String> {
    let json = read_plain_local(path)?;
    parse_json(&json)
}

/// Fetches JSON from a URL and extracts the text lines.
fn fetch_json_by_url(url: &str, key: &Option<String>) -> Result<String> {
    let json = fetch_plain_by_url(url, key)?;
    parse_json(&json)
}

/// Parses Scrapbox-style JSON and joins the lines with newlines.
fn parse_json(json: &str) -> Result<String> {
    let page: PageJson = serde_json::from_str(&json)?;
    let mut content = String::new();

    for line in page.lines {
        content.push_str(&line.text);
        content.push('\n');
    }

    Ok(content)
}
