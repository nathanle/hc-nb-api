use serde_json;
use serde::{Serialize};
use clap::Parser;
use rust_decimal::prelude::*;
use chrono::{NaiveDateTime, DateTime, Utc, Local, TimeZone};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    api_version: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct LkeCluster{
    id: u64,
    label: String,
    r#type: String,
    url: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct NodeBalancerListObject {
    client_conn_throttle: u64,
    created: String,
    hostname: String,
    id: u64,
    ipv4: String,
    ipv6: String,
    label: String,
    lke_cluster: LkeCluster,
    region: String,
    r#type: String,
    updated: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct NodeBalancerListData {
    data: Vec<NodeBalancerListObject>
}

fn round(c: &f64) -> f64 {
    let r = Decimal::from_f64(*c as f64)
        .unwrap()
        .round_dp(2)
        .to_f64()
        .unwrap();

    r
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
            let obj: NodeBalancerListObject = d;
            println!("{:#?}", obj);
        }
    } else {
        eprintln!("Request failed with status: {}", response.status());
    }

    Ok(())
}
