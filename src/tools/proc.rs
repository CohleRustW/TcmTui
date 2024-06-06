use crate::description::proc::{ClusterEelement, ProcTcmCenter, WorldElement};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum ProcAction {
    Start,
    Stop,
    Check,
    Restart,
    Auto,
    RunShell,
}

impl From<String> for ProcAction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Start" => ProcAction::Start,
            "Stop" => ProcAction::Stop,
            "Check" => ProcAction::Check,
            "Restart" => ProcAction::Restart,
            "Auto" => ProcAction::Auto,
            "RunShell" => ProcAction::RunShell,
            _ => panic!("Not support ProcAction"),
        }
    }
}

impl std::fmt::Display for ProcAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcAction::Start => write!(f, "Start"),
            ProcAction::Stop => write!(f, "Stop"),
            ProcAction::Check => write!(f, "Check"),
            ProcAction::Restart => write!(f, "Restart"),
            ProcAction::Auto => write!(f, "Auto"),
            ProcAction::RunShell => write!(f, "RunShell"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum ProcType {
    Cluster,
    World,
    Zone,
}

impl From<String> for ProcType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Cluster" => ProcType::Cluster,
            "World" => ProcType::World,
            "Zone" => ProcType::Zone,
            _ => panic!("Not support ProcType"),
        }
    }
}

impl std::fmt::Display for ProcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcType::Cluster => write!(f, "Cluster"),
            ProcType::World => write!(f, "World"),
            ProcType::Zone => write!(f, "Zone"),
        }
    }
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ProcInfo {
    pub layer: String,
    pub funcname: String,
    // TODO: 大小写
    pub group_name: String,
    pub func_id: i32,
    pub work_path: String,
    pub proc_name: String,
}

pub fn collect_proc_info(proc_center: ProcTcmCenter) -> Vec<ProcInfo> {
    let mut proc_group_name_map = HashMap::new();
    let mut procs: Vec<ProcInfo> = Vec::new();
    proc_center
        .procgroup_vec
        .iter()
        .for_each(|proc_group_info| {
            proc_group_info
                .proc_group_proc
                .iter()
                .for_each(|proc_info| {
                    proc_group_name_map
                        .insert(proc_info.func_name.clone(), proc_group_info.name.clone());
                });
        });
    proc_center.cluster_vec.iter().for_each(|cluster| {
        let base_work_path = cluster.work_path.clone();
        cluster.proc_list.iter().for_each(|element| match element {
            ClusterEelement::Proc(p) => {
                let proc_work_path= p.work_path.clone().unwrap_or("./".to_string());
                let proc_name: String = p.proc_name.clone().unwrap_or_else(|| p.func_name.clone());
                let work_path = join_work_path(&base_work_path, &proc_work_path);
                let proc_info = ProcInfo {
                    layer: LayerEnum::Cluster.into(),
                    funcname: p.func_name.clone(),
                    group_name: proc_group_name_map.get(&p.func_name).clone()
                    func_id: p.func_id,
                    proc_name: proc_name,
                    work_path,
                };
                procs.push(proc_info);
            }
            ClusterEelement::World(w) => {
                w.proc_list
                    .iter()
                    .for_each(|world_element| match world_element {
                        WorldElement::Proc(world_proc) => {
                            let world_proc_work_path = world_proc.work_path.clone().unwrap_or("./".to_string());
                            let work_path = join_work_path(&base_work_path, &world_proc_work_path);
                            let proc_name: String = world_proc
                                .proc_name
                                .clone()
                                .unwrap_or_else(|| world_proc.func_name.clone());
                            let proc_info = ProcInfo {
                                layer: LayerEnum::Cluster.into(),
                                funcname: world_proc.func_name.clone(),
                                group_name: proc_group_name_map
                                    .get(&world_proc.func_name)
                                    .unwrap()
                                    .clone(),
                                func_id: world_proc.func_id,
                                work_path: work_path,
                                proc_name: proc_name,
                            };
                            procs.push(proc_info);
                        }
                        WorldElement::Zone(zone_proc) => {
                            zone_proc.zone_proc_vec.iter().for_each(|zone_proc| {
                                let zone_work_path = zone_proc.work_path.clone().unwrap_or("./".to_string());
                                let work_path =
                                    join_work_path(&base_work_path, &zone_work_path);
                                let proc_name: String = zone_proc
                                    .proc_name
                                    .clone()
                                    .unwrap_or_else(|| zone_proc.func_name.clone());
                                let proc_info = ProcInfo {
                                    layer: LayerEnum::Zone.into(),
                                    funcname: zone_proc.func_name.clone(),
                                    group_name: proc_group_name_map
                                        .get(&zone_proc.func_name)
                                        .unwrap()
                                        .clone(),
                                    func_id: zone_proc.func_id,
                                    work_path,
                                    proc_name: proc_name,
                                };
                                procs.push(proc_info);
                            });
                        }
                    });
            }
        });
    });
    procs
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayerEnum {
    Cluster,
    Zone,
}
impl Into<String> for LayerEnum {
    fn into(self) -> String {
        match self {
            LayerEnum::Cluster => "Cluster".to_string(),
            LayerEnum::Zone => "Zone".to_string(),
        }
    }
}

fn join_work_path(work_path: &Option<String>, proc_name: &str) -> String {
    if let Some(p) = work_path {
        std::path::Path::new(&p)
            .join(proc_name)
            .to_str()
            .unwrap()
            .to_string()
    } else {
        std::path::Path::new("./")
            .join(proc_name)
            .to_str()
            .unwrap()
            .to_string()
    }
}
