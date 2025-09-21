use tokio_postgres::{Client, Error};
use std::collections::HashMap;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::env;
use serde::{Serialize};

#[derive(Debug)]
pub struct Nodebalancer {
    _id: i32,
    ip_address: String,
    port: i32,
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeBalancerListObject {
    client_conn_throttle: i32,
    created: String,
    hostname: String,
    pub id: i32,
    ipv4: String,
    ipv6: String,
    label: String,
    lke_cluster: LkeCluster,
    region: String,
    r#type: String,
    updated: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeObject {
    address: String,
    config_id: i32,
    id: i32,
    label: String,
    mode: String,
    nodebalancer_id: i32,
    status: String,
    weight: i32 
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct LkeCluster{
    id: i32,
    label: String,
    r#type: String,
    url: String,
}

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct NodeBalancerConfigObject {
    algorithm: String,
    check: String,
    check_attempts: i32,
    check_body: String,
    check_interval: i32,
    check_passive: bool,
    check_path: String,
    check_timeout: i32,
    cipher_suite: String,
    pub id: i32,
    pub nodebalancer_id: i32,
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
            nb_id INTEGER NOT NULL,
            ipv4 VARCHAR NOT NULL,
            region VARCHAR NOT NULL,
            lke_id INTEGER,
            PRIMARY KEY (nb_id)
            );
    ");

    match main_table.await {
        Ok(success) => println!("Nodebalancer table availabe"),
        Err(e) => println!("{:?}", e),
        }

    println!("{:#?}", nodebalancers);

    let update = connection.execute(
            "INSERT INTO nodebalancer (nb_id, ipv4, region, lke_id) VALUES ($1, $2, $3, $4)",
            &[&nodebalancers.id, &nodebalancers.ipv4, &nodebalancers.region, &nodebalancers.lke_cluster.id],
    ).await;

    match update {
        Ok(success) => println!("NB Row updated."),
        Err(e) => println!("{:?}", e),
        }

    Ok(())

}

pub async fn update_db_node(node: NodeObject) -> Result<(), Box<dyn std::error::Error>> {
    let mut node_connection = create_client().await;
    let node_table  = node_connection.batch_execute("
        CREATE TABLE IF NOT EXISTS node  (
            id INTEGER NOT NULL,
            address VARCHAR NOT NULL,
            status VARCHAR NOT NULL,
            nodebalancer_id INTEGER NOT NULL REFERENCES nodebalancer,
            PRIMARY KEY (id, nodebalancer_id)
            );
    ");
    match node_table.await {
        Ok(success) => println!("Node table availabe"),
        Err(e) => println!("{:?}", e),
        }

    let nb_table = node_connection.execute(
            "INSERT INTO node (id, address, status, nodebalancer_id) VALUES ($1, $2, $3, $4)",
            &[&node.id, &node.address, &node.status, &node.nodebalancer_id],
    ).await;

    Ok(())
}

pub async fn update_db_config(nodebalancer_config: NodeBalancerConfigObject) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_connection = create_client().await;
    let nb_cfg_conn  = config_connection.batch_execute("
        CREATE TABLE IF NOT EXISTS nodebalancer_config  (
            id INTEGER NOT NULL,
            algorithm VARCHAR NOT NULL,
            port INTEGER NOT NULL,
            up INTEGER,
            down INTEGER,
            nodebalancer_id INTEGER NOT NULL REFERENCES nodebalancer,
            PRIMARY KEY (id, nodebalancer_id)
            );
    ");
    println!("{:#?}", nodebalancer_config);
    println!("Done");

    let nb_cfg_table = config_connection.execute(
            "INSERT INTO nodebalancer_config (id, algorithm, port, up, down, nodebalancer_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&nodebalancer_config.id, &nodebalancer_config.algorithm, &nodebalancer_config.port, &nodebalancer_config.nodes_status.up, &nodebalancer_config.nodes_status.down, &nodebalancer_config.nodebalancer_id],
    ).await;

    match nb_cfg_conn.await {
        Ok(success) => println!("Nodebalancer config table availabe"),
        Err(e) => println!("{:?}", e),
        }

    match nb_cfg_table {
        Ok(success) => println!("Nodebalancer config row updated"),
        Err(e) => println!("Nodebalanacer config row error {:?}", e),
        }

    Ok(())


}
