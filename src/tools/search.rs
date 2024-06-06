use serde_json::Value;

pub fn search_vec<T>(hosts: &Vec<T>, keyword: &str) -> Vec<T>
where
    T: Clone + serde::Serialize,
{
    hosts
        .iter()
        .filter(|host| {
            let host_value: Value = serde_json::to_value(host).unwrap();
            host_value
                .as_object()
                .unwrap()
                .iter()
                .any(|(_, field_value)| {
                    if let Some(field_str) = field_value.as_str() {
                        field_str.contains(keyword)
                    } else if let Some(field_option) = field_value.as_object() {
                        field_option.get("Some").map_or(false, |inner_value| {
                            inner_value.as_str().unwrap().contains(keyword)
                        })
                    } else {
                        false
                    }
                })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::host::HostInfo;
    #[test]
    fn it_should_word() {
        let target_hosts = vec![
            HostInfo {
                inner_ip: "127.0.0.3".to_string(),
                host_name: "Host_DB_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "70".to_string(),
            },
            HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_DR_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "70".to_string(),
            },
            HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_DR_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "700".to_string(),
            },
            HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_Main_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "70".to_string(),
            },
            HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_Main_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "70".to_string(),
            },
            HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_Main_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "70".to_string(),
            },
        ];
        let keyword = "300";
        let filtered_hosts = search_vec(&target_hosts, keyword);
        assert_eq!(
            filtered_hosts,
            vec![]
        );
        let second_keyword = "700";
        let se_filtered_hosts = search_vec(&target_hosts, second_keyword);
        assert_eq!(
            se_filtered_hosts,
            vec![HostInfo {
                inner_ip: "127.0.0.1".to_string(),
                host_name: "Host_DR_70".to_string(),
                world_id: "4".to_string(),
                zone_id: "700".to_string(),
            }]
        )
    }
}
