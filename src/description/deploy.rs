use serde::Deserialize;
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct DeployTcmCenter {
    #[serde(rename = "ClusterDeploy")]
    pub cluster_deploy: ClusterDeploy,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct ClusterDeploy {
    #[serde(rename = "DeloyGroup")]
    pub deploy_groups: Vec<DeployGroup>,
    #[serde(rename = "world")]
    pub worlds: Vec<World>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct DeployGroup {
    #[serde(rename = "@Group")]
    pub group: String,
    #[serde(rename = "@Host")]
    pub host: Option<String>,
    #[serde(rename = "@InstID")]
    pub inst_id: Option<i32>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct World {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "zone")]
    pub zone_list: Vec<Zone>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Zone {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "DeloyGroup")]
    pub deploy_groups: Vec<ZoneDeployGroup>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct ZoneDeployGroup {
    #[serde(rename = "@Group")]
    pub group: String,
    #[serde(rename = "@Host")]
    pub host: String,
    #[serde(rename = "@CustomAttr")]
    pub custom_attr: Option<String>,
    #[serde(rename = "@InstID")]
    pub inst_id: Option<i32>,
}
