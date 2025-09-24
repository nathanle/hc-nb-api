use tokio_postgres::{Row, Client, Error};
use std::collections::HashMap;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::env;
use serde::{Serialize};
use std::sync::LazyLock;

static maindb_pw: LazyLock<String> = std::sync::LazyLock::new(|| { env::var("MAINDB_PASSWORD").expect("MAINDB_PASSWORD not set!") });
static maindb_hostport: LazyLock<String> = std::sync::LazyLock::new(|| { env::var("MAINDB_HOSTPORT").expect("MAINDB_HOSTPORT not set!") });

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
    lke_cluster: Option<LkeCluster>,
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

impl Default for LkeCluster {
    fn default() -> Self {
        LkeCluster {
            id: 0,
            label: String::new(),
            r#type: String::new(),
            url: String::new(),
        }
    }
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

pub async fn create_client() -> Client {
    let connector = create_connector().await;

    let url = format!("postgresql://akmadmin:{}@{}/defaultdb", maindb_pw.to_string(), maindb_hostport.to_string());
    let Ok((client, connection)) = tokio_postgres::connect(&url, connector).await else { panic!("Client failure.") };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client 

}

pub async fn db_init() -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(success) => println!("Nodebalancer table available"),
        Err(e) => println!("{:?}", e),
        }

    Ok(())

}

pub async fn update_db_nb(nodebalancers: NodeBalancerListObject) -> Result<(), Box<dyn std::error::Error>> {
    //println!( "{:#?}", nodebalancers);
    let mut connection = create_client().await;

    let update = connection.execute(
            "INSERT INTO nodebalancer (nb_id, ipv4, region, lke_id) VALUES ($1, $2, $3, $4)",
            &[&nodebalancers.id, &nodebalancers.ipv4, &nodebalancers.region, &nodebalancers.lke_cluster.unwrap_or_default().id],
    ).await;

    match update {
        Ok(success) => println!("NB Row updated."),
        Err(e) => {
            if e.to_string().contains("duplicate key value violates unique constraint") {
                ();
                //println!("{:?}", e);
            } else {
                println!("{:?}", e);
            }
        }
    }

    Ok(())

}

pub async fn get_nb_ids() -> Result<Vec<Row>, Error> {
    let mut node_connection = create_client().await;
    let nb_table = node_connection.query(
        "SELECT nb_id FROM nodebalancer", &[],
    ).await;

    Ok(nb_table?)

}
