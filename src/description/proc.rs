use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub enum ClusterEelement {
    #[serde(rename = "Proc")]
    Proc(Proc),
    #[serde(rename = "world")]
    World(World),
}
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Cluster {
    #[serde(rename = "@WorkPath")]
    pub work_path: Option<String>,
    #[serde(rename = "@AutoTimeGap")]
    pub auto_time_gap: Option<String>,
    #[serde(rename = "@OpTimeout")]
    pub op_timeout: Option<String>,
    #[serde(rename = "$value")]
    pub proc_list: Vec<ClusterEelement>,
}
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct ProcTcmCenter {
    #[serde(rename = "cluster")]
    pub cluster_vec: Vec<Cluster>,
    #[serde(rename = "ProcGroup")]
    pub procgroup_vec: Vec<ProcGroup>,
}
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct ProcGroupProc {
    #[serde(rename = "@FuncName")]
    pub func_name: String,
    #[serde(rename = "@Agrs")]
    pub agrs: Option<String>,
    #[serde(rename = "@GroupName")]
    pub group_name: Option<String>,
    #[serde(rename = "@Layer")]
    pub layer: Option<String>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct ProcGroup {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Layer")]
    pub layer: String,
    #[serde(rename = "Proc")]
    pub proc_group_proc: Vec<ProcGroupProc>
}
#[derive(Debug, PartialEq, Default, Deserialize, Clone)]
pub struct Proc {
    #[serde(rename = "@FuncName")]
    pub func_name: String,
    #[serde(rename = "@FuncID")]
    pub func_id: i32,
    #[serde(rename = "@ProcName")]
    pub proc_name: Option<String>,
    #[serde(rename = "@WorkPath")]
    pub work_path: Option<String>,
    #[serde(rename = "@Flag")]
    flag: String,
    #[serde(rename = "@IsCommon")]
    is_common: Option<String>,
    #[serde(rename = "@ConfigPath")]
    pub config_path: Option<String>,
    #[serde(rename = "@Seq")]
    seq: Option<String>,
    #[serde(rename = "@AutoScript")]
    auto_script: Option<String>,
    #[serde(rename = "@ReStartCmd")]
    restart_cmd: Option<String>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Clone)]
pub struct WorldProc {
    #[serde(rename = "@FuncName")]
    pub func_name: String,
    #[serde(rename = "@FuncID")]
    pub func_id: i32,
    #[serde(rename = "@ProcName")]
    pub proc_name: Option<String>,
    #[serde(rename = "@WorkPath")]
    pub work_path: Option<String>,
    #[serde(rename = "@Flag")]
    pub flag: String,
    #[serde(rename = "@IsCommon")]
    is_common: Option<String>,
    #[serde(rename = "@ConfigPath")]
    config_path: Option<String>,
    #[serde(rename = "@Seq")]
    seq: Option<String>,
    #[serde(rename = "@AutoScript")]
    auto_script: Option<String>,
    #[serde(rename = "@ReStartCmd")]
    restart_cmd: Option<String>,
}

// TODO: 这里的序列应该是无序的
#[derive(Debug, PartialEq, Deserialize)]
pub enum WorldElement {
    #[serde(rename = "Proc")]
    Proc(WorldProc),
    #[serde(rename = "zone")]
    Zone(Zone),
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct World {
    #[serde(rename = "@Isolated")]
    isolated: String,
    #[serde(rename = "@AutoTimeGap")]
    auto_time_gap: Option<String>,
    #[serde(rename = "@OpTimeout")]
    op_timeout: Option<String>,
    #[serde(rename = "$value")]
    pub proc_list: Vec<WorldElement>
}
#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Zone {
    #[serde(rename = "@Isolated")]
    isolated: String,
    #[serde(rename = "@AutoTimeGap")]
    auto_time_gap: Option<String>,
    #[serde(rename = "@OpTimeout")]
    op_timeout: Option<String>,
    #[serde(rename = "Proc")]
    pub zone_proc_vec: Vec<Proc>,
}
