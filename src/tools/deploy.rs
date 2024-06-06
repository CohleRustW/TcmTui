
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeployInfo {
    pub host_id: i32,
    pub group_name: String,
    pub inst_id: i32,
}

impl DeployInfo {
    pub fn new(host_id: i32, group_name: String, inst_id: i32) -> Self {
        DeployInfo {
            host_id,
            group_name,
            inst_id,
        }
    }
}

