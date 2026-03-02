use std::path::{Path, PathBuf};
use crate::{check_manifest, merge_files};
use getset::Getters;
use serde::{Deserialize, Serialize};
use crate::path::path::MODULE_PATH;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    #[default]
    Whitelist,
    Blacklist,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    #[default]
    Nfqws,
    Nfqws2,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApplistType {
    #[default]
    None,
    Blacklist,
    Whitelist,
}

#[derive(Default, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
#[serde(default)]
pub struct Config {
    service_type: ServiceType,
    active_lists: Vec<String>,
    active_ipsets: Vec<String>,
    active_exclude_lists: Vec<String>,
    active_exclude_ipsets: Vec<String>,
    list_type: ListType,
    strategy: String,
    strategy_nfqws2: String,
    app_list: ApplistType,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Manifest {
    schema: i32,
    name: String,
    author: String,
    description: String,
    dependencies: Vec<String>,
    file: String
}

impl ListType {
    /// # Returns
    ///
    /// (hostlist arg, ipset arg)
    pub async fn merge(&self, config: &Config) -> anyhow::Result<(String, String)> {
        let module_path_str = MODULE_PATH.to_str().unwrap();

        let (host_files, ipset_files, host_suffix, ipset_suffix, exclude_flag) = match self {
            ListType::Whitelist => (
                &config.active_lists,
                &config.active_ipsets,
                "hostlist",
                "ipset",
                "",
            ),
            ListType::Blacklist => (
                &config.active_exclude_lists,
                &config.active_exclude_ipsets,
                "hostlist-exclude",
                "ipset-exclude",
                "-exclude",
            ),
        };
        let host_paths: Vec<PathBuf> = host_files.iter()
            .map(|path| -> anyhow::Result<PathBuf> {
                let manifest = check_manifest(Path::new(path))?;
                Ok(PathBuf::from(manifest.file()))
            }).collect::<anyhow::Result<_>>()?;
        let ipset_paths: Vec<PathBuf> = ipset_files
            .iter()
            .map(|path| -> anyhow::Result<PathBuf> {
                let manifest = check_manifest(Path::new(path))?;
                Ok(PathBuf::from(manifest.file()))
            })
            .collect::<anyhow::Result<_>>()?;

        let host_path = MODULE_PATH.join(format!("tmp/{host_suffix}"));
        let ipset_path = MODULE_PATH.join(format!("tmp/{ipset_suffix}"));

        merge_files(&host_paths, host_path).await?;
        merge_files(&ipset_paths, ipset_path).await?;

        Ok((
            format!("--hostlist{exclude_flag}={module_path_str}/tmp/{host_suffix}"),
            format!("--ipset{exclude_flag}={module_path_str}/tmp/{ipset_suffix}"),
        ))
    }
}