use tokio_postgres::{Client, Error};
use std::collections::HashMap;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::env;
use serde::{Serialize};

#[derive(Debug)]
struct Nodebalancer {
    _id: i32,
    ip_address: String,
    port: i32,
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeBalancerListObject {
    client_conn_throttle: i32,
    created: String,
    hostname: String,
    id: i32,
    ipv4: String,
    ipv6: String,
    label: String,
    lke_cluster: LkeCluster,
    region: String,
    r#type: String,
    updated: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
struct LkeCluster{
    id: i32,
    label: String,
    r#type: String,
    url: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeBalancerConfig {
  algorithm: String,
  check: String,
  check_attempts: i32,
  check_body: String,
  check_interval: i32,
  check_passive: bool,
  check_path: String,
  check_timeout: i32,
  cipher_suite: String,
  id: i32,
  nodebalancer_id: i32,
  nodes_status: NodeStatus,
  port: i32,
  protocol: String,
  proxy_protocol: String,
  stickiness: String,
  udp_check_port: i32,
  udp_session_timeout: i32, 
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeStatus {
    down: i32,
    up: i32,
}

async fn create_connector() -> MakeTlsConnector {
    let mut builder = SslConnector::builder(SslMethod::tls()).expect("unable to create sslconnector builder");
    builder.set_ca_file("/tmp/ca.cert").expect("unable to load ca.cert");
    builder.set_verify(SslVerifyMode::NONE);
    let connector = MakeTlsConnector::new(builder.build());

    connector
} 

async fn create_client() -> Client {
    let connector = create_connector().await;
    let password = env::var("DB_PASSWORD");

    let url = format!("postgresql://akmadmin:{}@172.237.137.78:25079/defaultdb", password.expect("Password ENV var DB_PASSWORD not set."));
    let Ok((client, connection)) = tokio_postgres::connect(&url, connector).await else { todo!() };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client 

}

pub async fn update_db_nb(nodebalancers: NodeBalancerListObject) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = create_client().await;
    let main_table = connection.batch_execute("
        CREATE TABLE IF NOT EXISTS nodebalancer (
            ipv4 VARCHAR NOT NULL,
            region VARCHAR NOT NULL,
            nb_id INTEGER NOT NULL,
            lke_id INTEGER,
            PRIMARY KEY (ipv4)
            );
    ");

    match main_table.await {
        Ok(success) => println!("Nodebalancer table availabe"),
        Err(e) => println!("{:?}", e),
        }

    let node_table  = connection.batch_execute("
        CREATE TABLE IF NOT EXISTS node  (
            id SERIAL PRIMARY KEY,
            node VARCHAR NOT NULL,
            port INTEGER NOT NULL,
            state VARCHAR NOT NULL,
            nodebalancer_ipv4 VARCHAR NOT NULL REFERENCES nodebalancer 
            );
    ");
    match node_table.await {
        Ok(success) => println!("Nodebalancer table availabe"),
        Err(e) => println!("{:?}", e),
        }

    println!("{:#?}", nodebalancers);
    println!("Done");

    let update = connection.execute(
            "INSERT INTO nodebalancer (ipv4, region, nb_id, lke_id) VALUES ($1, $2, $3, $4)",
            &[&nodebalancers.ipv4, &nodebalancers.region, &nodebalancers.id, &nodebalancers.lke_cluster.id],
    ).await;

    match update {
        Ok(success) => println!("Row updated."),
        Err(e) => println!("{:?}", e),
        }

    Ok(())

}

pub async fn update_db_config(nodebalancer_config: NodeBalancerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = create_client().await;
    let nb_cfg_table  = connection.batch_execute("
        CREATE TABLE IF NOT EXISTS node  (
            id SERIAL PRIMARY KEY,
            algorithm VARCHAR NOT NULL,
            port INTEGER NOT NULL,
            nodebalancer_nb_id VARCHAR NOT NULL REFERENCES nodebalancer 
            );
    ");
    println!("{:#?}", nodebalancers);
    println!("Done");

    let nb_cfg_table = connection.execute(
            "INSERT INTO nodebalancer_config (algorithm, port, nodebalancer_nb_id) VALUES ($1, $2, $3)",
            &[&nodebalancers.algorithm, &nodebalancers.port, &nodebalancers.nodebalancer_id],
    ).await;

    match update {
        Ok(success) => println!("Row updated."),
        Err(e) => println!("{:?}", e),
        }

    Ok(())


}
