use crate::config::Manifest;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn prepare_manifests(input: &str, regex: &Regex, strategy_manifest: Option<&Manifest>, manifests: &HashMap<String, (PathBuf, Manifest)>, tmp_dir: &Path) -> anyhow::Result<String> {
    let required: HashSet<String> = regex.captures_iter(input).map(|c| c[1].to_string()).collect();
    let mut paths: HashMap<String, PathBuf> = HashMap::new();
    for id in &required {
        let manifest = manifests
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Manifest not found: {}", id))
            .and_then(|(path, m)| {
                let path = path.to_string_lossy();
                if let Some(strategy_manifest) = strategy_manifest {
                    if !strategy_manifest.dependencies().contains(&path.to_string()) {
                        return Err(anyhow::anyhow!("Dependency {} not declarated in strategy manifest", path))
                    }
                }
                Ok(m)
            })?;
        let path = Path::new(manifest.file());
        let mut dst = tmp_dir.join(id);
        if let Some(ext) = path.extension() {
            dst.set_extension(ext);
        }
        std::fs::copy(path, &dst)?;
        paths.insert(id.clone(), dst);
    }
    let result = regex.replace_all(input, |caps: &regex::Captures| {
        paths[&caps[1]].to_string_lossy().into_owned()
    });
    Ok(result.into_owned())
}