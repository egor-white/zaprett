use crate::{MODULE_PATH, merge_files};
use getset::Getters;
use serde::{Deserialize, Serialize};

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

impl ListType {
    /// # Returns
    ///
    /// (hostlist arg, ipset arg)
    pub async fn merge(&self, config: &Config) -> (String, String) {
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

        let host_path = MODULE_PATH.join(format!("tmp/{host_suffix}"));
        let ipset_path = MODULE_PATH.join(format!("tmp/{ipset_suffix}"));

        merge_files(host_files, host_path).await.unwrap();
        merge_files(ipset_files, ipset_path).await.unwrap();

        (
            format!("--hostlist{exclude_flag}={module_path_str}/tmp/{host_suffix}"),
            format!("--ipset{exclude_flag}={module_path_str}/tmp/{ipset_suffix}"),
        )
    }
}
