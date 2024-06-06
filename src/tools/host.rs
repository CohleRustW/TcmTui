use crate::database::TcmQueryResult;
use crate::description::*;
use crate::HOST_HASHMAP;
use serde::Deserialize;
use serde::Serialize;

use self::deploy::DeployTcmCenter;
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HostInfo {
    pub inner_ip: String,
    pub host_name: String,
    pub world_id: String,
    pub zone_id: String,
}

impl From<&TcmQueryResult> for HostInfo {
    fn from(tcm_query_result: &TcmQueryResult) -> Self {
        HostInfo {
            inner_ip: tcm_query_result.inner_ip.clone(),
            host_name: tcm_query_result.host_name.clone(),
            world_id: tcm_query_result.world_id.clone(),
            zone_id: tcm_query_result.zone_id.clone(),
        }
    }
}

pub fn collect_host_info(deploy_center: DeployTcmCenter) -> Vec<HostInfo> {
    let mut hosts: Vec<HostInfo> = Vec::new();
    for deploy_group in deploy_center.cluster_deploy.deploy_groups {
        let host_name: String;
        match deploy_group.host {
            Some(h) => host_name = h,
            None => host_name = "TcmHost".to_string(),
        };

        match HOST_HASHMAP.get(&host_name) {
            Some(inner_ip) => hosts.push(HostInfo {
                inner_ip: inner_ip.to_string(),
                host_name: host_name.to_string(),
                zone_id: "0".to_string(),
                world_id: "0".to_string(),
            }),
            None => panic!("配置文件解析错误"),
        };
    }
    for world in deploy_center.cluster_deploy.worlds {
        for zone in world.zone_list {
            for deploy_group in zone.deploy_groups {
                match HOST_HASHMAP.get(&deploy_group.host) {
                    Some(inner_ip) => hosts.push(HostInfo {
                        inner_ip: inner_ip.to_string(),
                        host_name: deploy_group.host,
                        zone_id: zone.id.clone(),
                        world_id: world.id.clone(),
                    }),
                    None => panic!("CONFIG FILE NOT EXCEPTE"),
                };
            }
        }
    }
    hosts
}