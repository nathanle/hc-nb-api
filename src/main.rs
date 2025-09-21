use serde_json;
use serde::{Serialize};
use clap::Parser;
use rust_decimal::prelude::*;
use chrono::{NaiveDateTime, DateTime, Utc, Local, TimeZone};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use crate::database::{
    update_db_node,
    update_db_nb,
    update_db_config,
    NodeBalancerListObject,
    NodeBalancerConfigObject,
    NodeObject
};

mod database;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    api_version: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct NodeBalancerListData {
    data: Vec<NodeBalancerListObject>,
    page: u64,
    pages: u64,
    results: u64,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct NodeBalancerConfigData {
    data: Vec<NodeBalancerConfigObject>,
    page: u64,
    pages: u64,
    results: u64,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct NodeListData {
    data: Vec<NodeObject>,
    page: u64,
    pages: u64,
    results: u64,
}

fn epoch_to_dt(e: &String) -> String {
    let timestamp = e.parse::<i64>().unwrap();
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");

    newdate.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let url = format!("https://api.linode.com/{}/nodebalancers", args.api_version);
    let auth_header = format!("Bearer {}", args.token);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header).unwrap());
    headers.insert("accept", HeaderValue::from_static("application/json"));

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client.get(url)
        .send()
        .await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let nbresult: NodeBalancerListData = serde_json::from_value(json.clone()).unwrap();
        for d in nbresult.data {
            let obj: database::NodeBalancerListObject = d;
            let nbid = obj.id;
            let _ = update_db_nb(obj).await;
            let config_url = format!("https://api.linode.com/{}/nodebalancers/{}/configs", args.api_version, nbid);
            println!("{:?}", config_url);
            let config_response = client.get(config_url)
                .send()
                .await?;

                if config_response.status().is_success() {
                    let json: serde_json::Value = config_response.json().await?;
                    let nbconfigdata: NodeBalancerConfigData = serde_json::from_value(json.clone()).unwrap();
                    for d in nbconfigdata.data {
                        let configobj: database::NodeBalancerConfigObject = d;
                        let borrow_configobj = &configobj;
                        let cfgid = borrow_configobj.id;
                        let nbid = borrow_configobj.nodebalancer_id;
                        let _ = update_db_config(configobj).await;
                        //Node Stuff
                        let node_url = format!("https://api.linode.com/{}/nodebalancers/{}/configs/{}/nodes", args.api_version, nbid, cfgid);
                        println!("{:?}", node_url);
                        let node_response = client.get(node_url)
                            .send()
                            .await?;

                        if node_response.status().is_success() {
                            let json: serde_json::Value = node_response.json().await?;
                            let nodedata: NodeListData = serde_json::from_value(json.clone()).unwrap();
                            for d in nodedata.data {
                                let nodeobj: database::NodeObject = d;
                                let _ = update_db_node(nodeobj).await;
                            }
                        }
                    }
                    println!("{:#?}", json);
                }
            //println!("{:#?}", obj);

        }
        println!("{:#?}", nbresult.pages);
    } else {
        eprintln!("Request failed with status: {}", response.status());
    }

    Ok(())
}
