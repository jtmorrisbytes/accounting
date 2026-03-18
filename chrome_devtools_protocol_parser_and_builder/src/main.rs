// this program downloads and generates rust code based on the chromedevtools protocol

use std::path::Path;

use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

const CHROME_DEV_TOOLS_URL: &str = "https://chromedevtools.github.io/devtools-protocol";

#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub struct CDFObjectProperties {
    name: String,
    description: Option<String>,
    #[serde(rename = "$ref")]
    external_refrence: Option<String>,
    #[serde(default)]
    optional: bool,
    r#type: Option<String>,
    items: Option<serde_json::Value>,
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDFCommands {
    name: String,
    #[serde(default)]
    decription: String,
    #[serde(default)]
    parameters: Vec<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDFEvents {
    name: String,
    description: String,
    #[serde(default)]
    experimental: bool,
    parameters: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDPDomainTypeDef {
    id: String,
    description: Option<String>,
    #[serde(rename = "type")]
    ty: String,
    // #[serde(default)]
    r#enum: Option<Vec<String>>,
    properties: Option<Vec<CDFObjectProperties>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDPVersion {
    major: String,
    minor: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDPDomain {
    domain: String,
    #[serde(default)]
    experimental: bool,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    types: Vec<CDPDomainTypeDef>,
    commands: Vec<CDFCommands>,
    #[serde(default)]
    events: Vec<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CDPSchema {
    version: CDPVersion,
    domains: Vec<CDPDomain>,
}
/// open file and download if not exists with cache
async fn ofadinewc<FilePath: AsRef<Path>, Url: IntoUrl>(
    p: FilePath,
    u: Url,
) -> Result<tokio::fs::File, anyhow::Error> {
    match tokio::fs::File::open(&p.as_ref()).await {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let r = reqwest::get(u).await?;
            dbg!(r.status());
            if !r.status().is_success() {
                return Err(anyhow::Error::msg("request failed"));
            }
            // write the file to the disk
            let mut f = tokio::fs::File::create(p).await?;
            tokio::io::copy(
                &mut tokio::io::BufReader::new(std::io::Cursor::new(r.bytes().await?)),
                &mut f,
            )
            .await?;
            f.flush().await.ok();
            f.seek(std::io::SeekFrom::Start(0)).await?;
            Ok(f)
        }
        Err(e) => return Err(e.into()),
    }
}

pub async fn parse_struct_from_json_file_cached<
    FilePath: AsRef<Path>,
    Url: IntoUrl,
    OutputType: DeserializeOwned,
>(
    f: FilePath,
    u: Url,
) -> Result<OutputType, anyhow::Error> {
    let f = ofadinewc(f, u).await?;
    let f = f
        .try_into_std()
        .map_err(|_e| anyhow::Error::msg("Failed to convert tokio::fs::File into std::io::File"))?;
    let mut reader = std::io::BufReader::new(f);
    let s: OutputType = serde_json::from_reader(&mut reader)?;
    Ok(s)
}

pub async fn parse_browser_protocol_from_json_file_cached() -> Result<CDPSchema, anyhow::Error> {
    let browserpath = format!(
        "{}/src/schemas/browser_protocol.json",
        env!("CARGO_MANIFEST_DIR")
    );
    parse_struct_from_json_file_cached(browserpath,"https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/refs/heads/master/json/browser_protocol.json").await
}

pub async fn parse_js_protocol_from_json_file_cached() -> Result<CDPSchema, anyhow::Error> {
    let browserpath = format!(
        "{}/src/schemas/js_protocol.json",
        env!("CARGO_MANIFEST_DIR")
    );
    parse_struct_from_json_file_cached(browserpath,"https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/refs/heads/master/json/js_protocol.json").await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s: CDPSchema = parse_browser_protocol_from_json_file_cached().await?;

    let tys = std::collections::HashMap::<String, String>::new();

    pub enum OProps {}

    pub enum TType {
        String,
        StringEnum,
        Object { properties: Vec<OProps> },
        Array,
        Integer,
        Number,
    }
    // types
    for d in s.domains {
        for t in d.types {
            let key = format!("{}.{}", d.domain, t.id);
            match t.ty.as_str() {
                "string" => {
                    println!("{key} ty=string")
                }
                "object" => {
                    println!("{key} ty=object");
                    for p in t.properties.unwrap_or_default() {
                        println!(
                            "\tprop: {}, type: '{}', ref: '{}' optional:{}",
                            p.name,
                            p.r#type.unwrap_or_default(),
                            p.external_refrence.unwrap_or_default(),
                            p.optional,
                        );
                    }
                }
                "array" => {
                    println!("{key} ty=array");
                }
                "integer" => {
                    println!("{key} t=integer");
                }
                "number" => {
                    println!("{key} t=number");
                }
                "enum" => {
                    panic!("enum!")
                }
                u @ _ => {
                    println!("unknown type {u}")
                }
            }

            // dbg!(key,t.ty);
        }
    }
    // let j = parse_js_protocol_from_json_file_cached().await?;
    // flush and return to the beginning of the file
    // let s : CDPSchema = serde_json::from_reader(&mut std::io::BufReader::new(&f))?;
    // dbg!(s);

    // let json: serde_json::Value = r.json().await?;

    // let schema: CDPSchema = serde_json::from_value(json.clone())?;
    // dbg!(schema);

    // let r = reqwest::get(format!("https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/refs/heads/master/json/js_protocol.json")).await?;

    Ok(())
}
