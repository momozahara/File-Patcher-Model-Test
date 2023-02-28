use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
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

fn main() -> Result<(), io::Error> {
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

    let hash = get_hash(file_path).unwrap();

    if !hash.eq(&latest_hash) {
        println!("Hash Mismatch!");
        fs::remove_file(RAW_PATH).unwrap();
        let mut iter = data.assets.iter();
        if let Some(asset) = iter.next() {
            if asset.name.contains("Kubernetes.Object.Generator") && asset.name.contains("msi") {
                download(&asset.browser_download_url).unwrap();
            }
        }
    } else {
        println!("Hash Match!");
    }

    Ok(())
}

fn get_latest() -> Result<reqwest::blocking::Response, io::Error> {
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

    let mut file = fs::File::create(RAW_PATH).unwrap();
    io::copy(&mut response, &mut file).unwrap();

    Ok(())
}

fn get_hash(file_path: &Path) -> Result<String, io::Error> {
    let hash = sha256::try_digest(file_path).unwrap();
    Ok(hash)
}
