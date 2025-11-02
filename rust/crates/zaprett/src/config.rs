use crate::{MODULE_PATH, merge_files};
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    Whitelist,
    Blacklist,
}

impl Default for ListType {
    fn default() -> Self {
        Self::Whitelist
    }
}

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Config {
    #[serde(default)]
    active_lists: Vec<String>,
    #[serde(default)]
    active_ipsets: Vec<String>,
    #[serde(default)]
    active_exclude_lists: Vec<String>,
    #[serde(default)]
    active_exclude_ipsets: Vec<String>,
    #[serde(default)]
    list_type: ListType,
    #[serde(default)]
    strategy: String,
    #[serde(default)]
    app_list: String,
    #[serde(default)]
    whitelist: Vec<String>,
    #[serde(default)]
    blacklist: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active_lists: vec![],
            active_ipsets: vec![],
            active_exclude_lists: vec![],
            active_exclude_ipsets: vec![],
            list_type: Default::default(),
            strategy: String::new(),
            app_list: String::new(),
            whitelist: vec![],
            blacklist: vec![],
        }
    }
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
