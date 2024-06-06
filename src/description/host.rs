use serde::Deserialize;

fn return_0() -> i32 {
    0
}
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Host {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@InnerIP")]
    pub inner_ip: String,
    #[serde(rename = "@OuterIPCount")]
    #[serde(default = "return_0")]
    pub outer_ip_count: i32,
    #[serde(rename = "OuterIP")]
    pub outer_ip: Option<String>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct HostTcmCenter {
    #[serde(rename = "HostTab")]
    pub host_tab: HostTab,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct HostTab {
    #[serde(rename= "Host")]
    pub hosts: Vec<Host>
}