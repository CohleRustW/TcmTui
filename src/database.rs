use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};

use crate::tools::{deploy::DeployInfo, host::HostInfo, proc::ProcInfo};
use sqlx::Row;
use tracing::log::info;

#[cfg(target_os = "windows")]
pub(crate) const SQLX_DATABASE_URL: &str = "C:\\sqlite:tcm.db";
#[cfg(not(target_os = "windows"))]
pub(crate) const SQLX_DATABASE_URL: &str = "/tmp/sqlite:tcm.db";

pub struct TcmQuery {
    pub world_id: String,
    pub zone_id: String,
    pub proc_id: String,
    pub inst_id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TcmQueryResult {
    pub host_id: i32,
    pub inner_ip: String,
    pub host_name: String,
    pub world_id: String,
    pub zone_id: String,
    pub func_id: i32,
    pub proc_type: String,
    pub work_path: String,
    pub func_name: String,
    pub proc_name: String,
    pub proc_group_name: String,
    pub inst_id: i32,
}

pub async fn init_sqlx_table() -> Result<SqlitePool, sqlx::Error> {
    info!("init sqlx");
    std::fs::File::create(SQLX_DATABASE_URL)?;
    let options = SqliteConnectOptions::new().filename(SQLX_DATABASE_URL);
    let pool = SqlitePool::connect_with(options).await?;
    for sql in return_init_sqls() {
        sqlx::query(sql).execute(&pool).await?;
    }
    Ok(pool)
}

fn return_init_sqls() -> Vec<&'static str> {
    vec![
        "CREATE TABLE hosts (
            id INTEGER PRIMARY KEY,
            inner_ip TEXT NOT NULL,
            host_name TEXT NOT NULL,
            world_id TEXT NOT NULL,
            zone_id TEXT NOT NULL
        )",
        "CREATE TABLE procs (
            func_id INTEGER PRIMARY KEY,
            proc_type TEXT NOT NULL,
            work_path TEXT NOT NULL,
            func_name TEXT NOT NULL,
            proc_name TEXT NOT NULL,
            proc_group_name TEXT NOT NULL
        )",
        "CREATE TABLE deploy (
            id INTEGER PRIMARY KEY,
            host_id INTEGER NOT NULL,
            group_name TEXT NOT NULL,
            inst_id INTEGER NOT NULL,
            UNIQUE(host_id, group_name, inst_id)
        )",
    ]
}

pub async fn insert_hosts(pool: &SqlitePool, hosts: &Vec<HostInfo>) -> Result<(), sqlx::Error> {
    let mut values = String::new();

    for (i, host_info) in hosts.iter().enumerate() {
        if i > 0 {
            values.push_str(", ");
        }
        values.push_str(&format!(
            "('{}', '{}', '{}', '{}')",
            host_info.inner_ip, host_info.host_name, host_info.world_id, host_info.zone_id
        ));
    }

    let sql = format!(
        "INSERT INTO hosts (inner_ip, host_name, world_id, zone_id) VALUES {}",
        values
    );
    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}

pub async fn insert_procs(pool: &SqlitePool, procs: &Vec<ProcInfo>) -> Result<(), sqlx::Error> {
    let mut values = String::new();

    for (i, proc) in procs.iter().enumerate() {
        if i > 0 {
            values.push_str(", ");
        }
        values.push_str(&format!(
            "('{}', '{}', '{}', '{}', '{}', '{}')",
            proc.func_id, proc.layer, proc.work_path, proc.funcname, proc.proc_name, proc.group_name
        ));
    }

    let sql = format!(
        "INSERT INTO procs (func_id, proc_type, work_path, func_name, proc_name, proc_group_name) VALUES {}",
        values
    );

    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}
pub async fn insert_deploy(pool: &SqlitePool, procs: &Vec<DeployInfo>) -> Result<(), sqlx::Error> {
    let mut values = String::new();

    for (i, proc) in procs.iter().enumerate() {
        if i > 0 {
            values.push_str(", ");
        }
        values.push_str(&format!(
            "('{}', '{}', '{}')",
            proc.host_id, proc.group_name, proc.inst_id
        ));
    }

    let sql = format!(
        "INSERT INTO deploy (host_id, group_name, inst_id) VALUES {}",
        values
    );

    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}

pub async fn select_all_proc(pool: &SqlitePool) -> Result<Vec<ProcInfo>, sqlx::Error> {
    let sql: &str = "select * from procs";
    let query = sqlx::query(sql).fetch_all(pool).await?;
    let mut procs = Vec::new();
    for q in query {
        let c = ProcInfo::from_row(&q).unwrap();
        procs.push(c)
    }
    Ok(procs)
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for HostInfo {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let inner_ip = row.try_get("inner_ip")?;
        let host_name = row.try_get("host_name")?;
        let world_id = row.try_get("world_id")?;
        let zone_id = row.try_get("zone_id")?;
        Ok(Self {
            inner_ip,
            world_id,
            host_name,
            zone_id,
        })
    }
}

pub async fn select_all_host(pool: &SqlitePool) -> Result<Vec<HostInfo>, sqlx::Error> {
    let sql: &str = "select * from hosts";
    let query = sqlx::query(sql).fetch_all(pool).await?;
    let mut hosts = Vec::new();
    for q in query {
        let c = HostInfo::from_row(&q).unwrap();
        hosts.push(c)
    }
    Ok(hosts)
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for ProcInfo {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let layer = row.try_get("proc_type")?;
        let funcname = row.try_get("func_name")?;
        let group_name = row.try_get("proc_group_name")?;
        let func_id = row.try_get("func_id")?;
        let work_path = row.try_get("work_path")?;
        let proc_name = row.try_get("proc_name")?;
        Ok(Self {
            layer,
            funcname,
            group_name,
            func_id,
            work_path,
            proc_name,
        })
    }
}
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for TcmQueryResult {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let host_id = row.try_get("host_id")?;
        let inner_ip = row.try_get("inner_ip")?;
        let host_name = row.try_get("host_name")?;
        let world_id = row.try_get("world_id")?;
        let zone_id = row.try_get("zone_id")?;
        let func_id = row.try_get("func_id")?;
        let proc_type = row.try_get("proc_type")?;
        let work_path = row.try_get("work_path")?;
        let func_name = row.try_get("func_name")?;
        let proc_name = row.try_get("proc_name")?;
        let proc_group_name = row.try_get("proc_group_name")?;
        let inst_id = row.try_get("inst_id")?;
        Ok(Self {
            host_id,
            inner_ip,
            host_name,
            world_id,
            zone_id,
            func_id,
            proc_type,
            work_path,
            func_name,
            proc_name,
            proc_group_name,
            inst_id,
        })
    }
}

pub fn query_sql_join(sql: &str, table_name: &str) -> String {
    let mut query_vec: Vec<&str> = sql.trim_end_matches('.').split(".").collect();
    if query_vec.len() < 4 {
        let missing_parts = 4 - query_vec.len();
        query_vec.extend(vec!["*"; missing_parts]);
    }
    let query = TcmQuery {
        world_id: query_vec[0].to_string(),
        zone_id: query_vec[1].to_string(),
        proc_id: query_vec[2].to_string(),
        inst_id: query_vec[3].to_string(),
    };
    let mut modify_sql = format!("select * from {}", table_name);
    if query.proc_id != "*" {
        modify_sql = format!(
            "{} JOIN procs ON procs.proc_group_name = deploy.group_name and procs.func_id = '{}'",
            modify_sql, query.proc_id
        );
    } else {
        modify_sql = format!("{} JOIN procs", modify_sql);
    }
    if query.world_id != "*" && query.zone_id != "*" {
        modify_sql = format!(
            "{} JOIN deploy ON hosts.world_id = '{}' and hosts.zone_id = '{}'",
            modify_sql, query.world_id, query.zone_id
        );
    } else if query.world_id != "*" {
        modify_sql = format!(
            "{} JOIN deploy ON hosts.world_id = '{}'",
            modify_sql, query.world_id
        );
    } else if query.zone_id != "*" {
        modify_sql = format!(
            "{} JOIN deploy ON hosts.zone_id = '{}'",
            modify_sql, query.zone_id
        );
    } else {
        modify_sql = format!("{} JOIN deploy", modify_sql);
    }
    if query.inst_id != "*" {
        modify_sql = format!(
            "{} WHERE deploy.inst_id = '{}' and hosts.id = deploy.host_id",
            modify_sql, query.inst_id
        );
    } else {
        modify_sql = format!("{} WHERE hosts.id = deploy.host_id", modify_sql);
    }
    modify_sql
}

pub async fn query_hosts_sql(sql: &str, db: &SqlitePool) -> anyhow::Result<Vec<TcmQueryResult>> {
    let sql = query_sql_join(sql, "hosts");
    let query = sqlx::query(&sql).fetch_all(db).await?;
    let mut hosts = Vec::new();
    for f in query {
        let c = TcmQueryResult::from_row(&f).unwrap();
        hosts.push(c);
    }
    Ok(hosts)
}

pub async fn get_host_id(
    pool: &SqlitePool,
    inner_ip: &str,
    world_id: &str,
    zone_id: &str,
) -> Result<i32, sqlx::Error> {
    let sql = format!(
        "select id from hosts WHERE inner_ip = '{}' and world_id = '{}' and zone_id = '{}';",
        inner_ip, world_id, zone_id
    );
    let row = sqlx::query(&sql).fetch_one(pool).await?;
    Ok(row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_sql_parse() {
        // let db = init_sqlx_table().await.unwrap();
        // init_data(&db).await;
        let tcm_query = "*.*.*.*";
        let sql = query_sql_join(tcm_query, "hosts");
        assert_eq!(
            sql,
            "select * from hosts JOIN procs JOIN deploy WHERE hosts.id = deploy.host_id"
        );
        let tcm_world_query = "2.*.*.*";
        let sql1 = query_sql_join(tcm_world_query, "hosts");
        assert_eq!(
            sql1,
            "select * from hosts JOIN procs JOIN deploy ON hosts.world_id = '2' WHERE hosts.id = deploy.host_id"
        );
        let tcm_zone_query = "*.200.*.*";
        let sql2 = query_sql_join(tcm_zone_query, "hosts");
        assert_eq!(
            sql2,
            "select * from hosts JOIN procs JOIN deploy ON hosts.zone_id = '200' WHERE hosts.id = deploy.host_id"
        );
        let tcm_zone_func_id_query = "*.200.201.*";
        let sql3 = query_sql_join(tcm_zone_func_id_query, "hosts");
        assert_eq!(
            sql3,
            "select * from hosts JOIN procs ON procs.proc_group_name = deploy.group_name and procs.func_id = '201' JOIN deploy ON hosts.zone_id = '200' WHERE hosts.id = deploy.host_id"
        );
        let tcm_zone_func_inst_id_query = "*.200.201.1";
        let sql4 = query_sql_join(tcm_zone_func_inst_id_query, "hosts");
        assert_eq!(
            sql4,
            "select * from hosts JOIN procs ON procs.proc_group_name = deploy.group_name and procs.func_id = '201' JOIN deploy ON hosts.zone_id = '200' WHERE deploy.inst_id = '1' and hosts.id = deploy.host_id"
        );
    }
}
