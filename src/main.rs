use clap::Parser;
use quick_xml::de::from_str;
mod app;
mod components;
mod config;
mod database;
mod description;
mod event;
mod tools;
pub mod ui;
mod utils;
#[macro_use]
extern crate lazy_static;
use app::start_app;
use database::init_sqlx_table;
use hashbrown::HashMap;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use utils::{drop_app, init_data};
lazy_static! {
    static ref HOST_HASHMAP: HashMap<String, String> = {
        let args = utils::Args::parse();
        let mut m = HashMap::new();
        let xml = args.config_path.join("host.xml");
        let host_xml: String = std::fs::read_to_string(xml).unwrap();
        let host_xml_content: description::host::HostTcmCenter = from_str(&host_xml).unwrap();
        for host_info in host_xml_content.host_tab.hosts {
            // TODO: 这里 IP 应该是唯一的 对应多个 Name
            m.insert(host_info.name, host_info.inner_ip);
        }
        m.insert("TcmHost".to_string(), "127.0.0.1".to_string());
        m
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = utils::Args::parse();
    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(if args.debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        }))
        .init();
    let db = init_sqlx_table().await?;
    if let Err(e) = init_data(&db, args.config_path).await {
        error!("Init data filed, error ->[{}]", e);
        drop_app();
    }

    // UI
    start_app(&db).await?;

    // drop resouce
    Ok(())
}
