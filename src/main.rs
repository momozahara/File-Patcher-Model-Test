use data_encoding::HEXUPPER;
use reqwest::header::{ACCEPT, USER_AGENT};
use ring::digest::{Context, Digest, SHA256};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs::File;
use std::io::{BufReader, Read};
use std::{fs, io, path::Path};

static RAW_PATH: &str = "./Kubernetes.Object.Generator.msi";

#[derive(Deserialize, Serialize, Debug)]
struct LatestRelease {
    assets: Vec<Asset>,
    body: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn main() -> io::Result<()> {
    let file_path = Path::new(RAW_PATH);

    let response = get_latest().unwrap();
    let data: LatestRelease = from_str(&response.text().unwrap()).unwrap();

    if !file_path.exists() {
        println!("Not Found");
        let mut iter = data.assets.iter();
        if let Some(asset) = iter.next() {
            if asset.name.contains("Kubernetes.Object.Generator") && asset.name.contains("msi") {
                download(&asset.browser_download_url).unwrap();
            }
        }

        return Ok(());
    }

    let data_n: Vec<&str> = data.body.split('\n').collect();

    let mut latest_hash = String::new();
    let mut iter = data_n.iter();
    while let Some(data) = iter.next() {
        let data_n: Vec<&str> = data.split_whitespace().collect();
        if data_n.len() > 1 {
            if data_n[1].contains("Kubernetes.Object.Generator") && data_n[1].contains("msi") {
                latest_hash = data_n[0].to_string();
            }
        }
    }

    let input = File::open(&file_path).unwrap();
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader).unwrap();
    let hash = HEXUPPER.encode(digest.as_ref()).to_lowercase();

    if !hash.eq(&latest_hash) {
        println!("Hash Mismatch!");
        fs::remove_file(RAW_PATH)?;
        let mut iter = data.assets.iter();
        if let Some(asset) = iter.next() {
            if asset.name.contains("Kubernetes.Object.Generator") && asset.name.contains("msi") {
                download(&asset.browser_download_url)?;
            }
        }
    } else {
        println!("Hash Match!");
    }

    Ok(())
}

fn get_latest() -> io::Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    let request = client
        .get("https://api.github.com/repos/momozahara/kubernetes-object-generator/releases/latest")
        .header(USER_AGENT, "rust")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .unwrap();
    Ok(request)
}

fn download(url: &str) -> io::Result<()> {
    println!("Download {url}");

    let mut response = reqwest::blocking::get(url).unwrap();

    let mut file = fs::File::create(RAW_PATH)?;
    io::copy(&mut response, &mut file)?;

    Ok(())
}

/// source: https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html#calculate-the-sha-256-digest-of-a-file
fn sha256_digest<R: Read>(mut reader: R) -> io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}
