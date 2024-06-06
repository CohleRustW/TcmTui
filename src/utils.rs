use anyhow::anyhow;
use clap::Parser;
use core::panic;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;


use crate::{
    database::{get_host_id, insert_deploy, insert_hosts, insert_procs, SQLX_DATABASE_URL},
    description::{deploy::DeployTcmCenter, host::HostTcmCenter, proc::ProcTcmCenter},
    tools::{
        deploy::DeployInfo,
        host::{collect_host_info, HostInfo},
        proc::{collect_proc_info, ProcInfo},
    },
    HOST_HASHMAP,
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TcmCommand {
    ListProc,
    Start,
    Stop,
    CheckBe,
    CheckNo,
}
impl Into<String> for TcmCommand {
    fn into(self) -> String {
        match self {
            TcmCommand::ListProc => "listProc".to_string(),
            TcmCommand::Start => "start".to_string(),
            TcmCommand::Stop => "stop".to_string(),
            TcmCommand::CheckBe => "checkbe".to_string(),
            TcmCommand::CheckNo => "checkno".to_string(),
        }
    }
}

#[warn(unused)]
pub enum ParseType {
    #[allow(dead_code)]
    HOST,
    #[allow(dead_code)]
    PROC,
    DEPLOY,
}

pub enum TcmCenterType {
    Deploy(DeployTcmCenter),
    Host(HostTcmCenter),
    Proc(ProcTcmCenter),
}

impl Into<DeployTcmCenter> for TcmCenterType {
    fn into(self) -> DeployTcmCenter {
        match self {
            TcmCenterType::Deploy(deploy) => deploy,
            _ => panic!("Expected a DeployTcmCenter, but got a different TcmCenter variant"),
        }
    }
}

impl Into<ProcTcmCenter> for TcmCenterType {
    fn into(self) -> ProcTcmCenter {
        match self {
            TcmCenterType::Proc(proc) => proc,
            _ => panic!("Expected a ProcTcmCenter, but got a different TcmCenter variant"),
        }
    }
}

impl Into<HostTcmCenter> for TcmCenterType {
    fn into(self) -> HostTcmCenter {
        match self {
            TcmCenterType::Host(host) => host,
            _ => panic!("Expected a HostTcmCenter, but got a different TcmCenter variant"),
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
pub struct Args {
    /// Number of times to greet
    #[arg(short='c', long, default_value = "./")]
    pub config_path: PathBuf,
    #[arg(short= 'd', default_value= "false")]
    pub debug: bool,
}

pub fn return_parsed_center(
    parse_type: ParseType,
    path: &PathBuf,
) -> anyhow::Result<TcmCenterType> {
    let depoloy_xml = path.join("procdeploy.xml");
    let host_xml = path.join("host.xml");
    let proc_xml = path.join("proc.xml");
    let v = vec![&depoloy_xml, &host_xml, &proc_xml];
    for xml in v {
        if !xml.exists() {
            return Err(anyhow!(
                "Config file -> {} not exists!",
                xml.to_str().unwrap().to_string()
            ));
        }
    }
    match parse_type {
        ParseType::DEPLOY => {
            let xml = std::fs::read_to_string(&depoloy_xml)?;
            Ok(TcmCenterType::Deploy(from_str(&xml)?))
        }
        ParseType::HOST => {
            let xml = std::fs::read_to_string(host_xml)?;
            Ok(TcmCenterType::Host(from_str(&xml)?))
        }
        ParseType::PROC => {
            let xml = std::fs::read_to_string(proc_xml)?;
            Ok(TcmCenterType::Proc(from_str(&xml)?))
        }
    }
}

pub fn return_hosts(path: &PathBuf) -> anyhow::Result<Vec<HostInfo>> {
    let center: DeployTcmCenter = return_parsed_center(ParseType::DEPLOY, path)?.into();
    Ok(collect_host_info(center))
}
pub fn return_procs(path: &PathBuf) -> anyhow::Result<Vec<ProcInfo>> {
    let center: ProcTcmCenter = return_parsed_center(ParseType::PROC, path)?.into();
    let procs = collect_proc_info(center);
    Ok(procs)
}

pub async fn return_deploy(db: &SqlitePool, path: &PathBuf) -> anyhow::Result<Vec<DeployInfo>> {
    let center: DeployTcmCenter = return_parsed_center(ParseType::DEPLOY, path)?.into();
    match collect_deploy_info(center, db).await {
        Ok(result) => Ok(result),
        Err(e) => {
            return Err(anyhow!("Parse config xml failed! -> [{}]", e));
        }
    }
}

pub async fn collect_deploy_info(
    center: DeployTcmCenter,
    db: &SqlitePool,
) -> Result<Vec<DeployInfo>, sqlx::Error> {
    let mut deploy_info = Vec::new();
    for deploy in center.cluster_deploy.deploy_groups {
        let inner_ip: String;
        let world_id: String = 0.to_string();
        let zone_id: String = 0.to_string();

        match deploy.host {
            Some(host) => {
                inner_ip = HOST_HASHMAP.get(&host).unwrap().clone();
            }
            None => {
                inner_ip = "localhost".to_string();
            }
        }
        let inst_id = deploy.inst_id.unwrap_or_else(|| 0);
        let host_id = get_host_id(db, &inner_ip, &world_id, &zone_id).await?;
        deploy_info.push(DeployInfo::new(host_id, deploy.group, inst_id));
    }
    for deploy in center.cluster_deploy.worlds {
        let world_id = deploy.id;
        for zone in deploy.zone_list {
            for deploy in zone.deploy_groups {
                let inner_ip = HOST_HASHMAP.get(&deploy.host).unwrap().clone();
                let inst_id = deploy.inst_id.unwrap_or_else(|| 1);
                let host_id = get_host_id(db, &inner_ip, &world_id, &zone.id).await?;
                deploy_info.push(DeployInfo::new(host_id, deploy.group, inst_id));
            }
        }
    }
    Ok(deploy_info)
}

pub fn drop_app() {
    // with drop all resource with database
    if std::path::Path::new(super::database::SQLX_DATABASE_URL).exists() {
        std::fs::remove_file(super::database::SQLX_DATABASE_URL).unwrap();
        debug!("drop database file {}", SQLX_DATABASE_URL);
    }
    std::process::exit(1);
}

pub async fn init_data(db: &SqlitePool, path: PathBuf) -> anyhow::Result<()> {
    let hosts = return_hosts(&path)?;
    insert_hosts(db, &hosts).await?;
    let procs = return_procs(&path)?;
    insert_procs(db, &procs).await?;
    let deploys = return_deploy(db, &path).await?;
    insert_deploy(db, &deploys).await?;
    debug!("insert data done");
    Ok(())
}

pub fn tabs_to_spaces(input: String) -> String {
    if input.contains('\t') {
        input.replace('\t', "  ")
    } else {
        input
    }
}

pub fn trim_offset(src: &str, mut offset: usize) -> &str {
    let mut start = 0;
    for c in UnicodeSegmentation::graphemes(src, true) {
        let w = c.width();
        if w <= offset {
            offset -= w;
            start += c.len();
        } else {
            break;
        }
    }
    &src[start..]
}

pub fn file_content(path: &std::path::Path) -> anyhow::Result<String> {
    let s_p = path.to_str().unwrap().to_string();
    if !path.exists() {
        return Err(anyhow!("File -> [{}] not exist", s_p));
    }
    if is_binary_file(path)? {
        return Err(anyhow!("File -> [{}] is binary! \nCan't read!", s_p));
    }
    let file = std::fs::read(path)?;
    let content = String::from_utf8_lossy(&file).to_string();
    Ok(content)
}

fn is_binary_file<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
    let file_type = infer::get_from_path(path)?;

    Ok(file_type.map_or(false, |ftype| !ftype.mime_type().starts_with("text/")))
}
