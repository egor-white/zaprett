use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use anyhow::bail;
use regex::Regex;
use crate::config::Manifest;

pub fn prepare_manifests(input: &str, regex: &Regex, manifests: &HashMap<String, Manifest>, tmp_dir: &Path) -> anyhow::Result<String> {
    let required: HashSet<String> = regex.captures_iter(input).map(|c| c[1].to_string()).collect();
    for name in &required {
        if !manifests.contains_key(name) {
            bail!("Manifest not found: {}", name)
        }
    }
    let mut paths: HashMap<String, PathBuf> = HashMap::new();
    for name in &required {
        let manifest = &manifests[name];
        let path = Path::new(manifest.file());
        let dst = tmp_dir.join(format!("{}.txt", name));
        std::fs::copy(path, &dst)?;
        paths.insert(name.clone(), dst);
    }
    let result = regex.replace_all(input, |caps: &regex::Captures| {
        paths[&caps[1]].to_string_lossy().into_owned()
    });
    Ok(result.into_owned())
}