use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Value {
    first: bool,
}

impl Default for Value {
    fn default() -> Self {
        Self { first: true }
    }
}

fn path() -> Result<PathBuf> {
    let base = std::env::var("APPDATA").context("appdata")?;
    Ok(PathBuf::from(base).join("unixish").join("state.json"))
}

fn load() -> Result<Value> {
    let file = path()?;
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    if !file.exists() {
        let value = Value::default();
        save(&value)?;
        return Ok(value);
    }
    let text = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&text).unwrap_or_default())
}

fn save(value: &Value) -> Result<()> {
    let file = path()?;
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(file, text)?;
    Ok(())
}

pub fn first() -> bool {
    if let Ok(mut value) = load() {
        if value.first {
            value.first = false;
            let _ = save(&value);
            return true;
        }
    }
    false
}
