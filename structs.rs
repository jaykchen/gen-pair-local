use serde::{ Deserialize, Serialize };
use serde_json::Result;

#[derive(Debug, Deserialize, Serialize)]
struct PandocData {
    pandoc_api_version: Vec<u32>,
    meta: serde_json::Value,
    blocks: Vec<Block>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Block {
    t: String,
    c: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct Header {
    level: u32,
    content: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Str {
    c: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Space {}

#[derive(Debug, Deserialize, Serialize)]
struct Para {
    c: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BulletList {
    c: Vec<Vec<Plain>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Plain {
    t: String,
    c: Vec<serde_json::Value>,
}

fn main() -> Result<()> {}
