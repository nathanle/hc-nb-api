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
    id: u64,
    label: String,
    r#type: String,
    url: String,
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

pub async fn update_db(nodebalancers: NodeBalancerListObject) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = create_client().await;
    let main_table = connection.batch_execute("
        CREATE TABLE IF NOT EXISTS nodebalancer (
            id SERIAL PRIMARY KEY,
            ipv4 VARCHAR NOT NULL,
            region VARCHAR NOT NULL,
            nb_id INTEGER NOT NULL
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
            nodebalancer_id INTEGER NOT NULL REFERENCES nodebalancer 
            );
    ");
    match node_table.await {
        Ok(success) => println!("Nodebalancer table availabe"),
        Err(e) => println!("{:?}", e),
        }

    println!("{:#?}", nodebalancers);
    println!("Done");

    let _ = connection.execute(
            "INSERT INTO nodebalancer (ipv4, region, nb_id) VALUES ($1, $2, $3)",
            &[&nodebalancers.ipv4, &nodebalancers.region, &nodebalancers.id],
    ).await;

    Ok(())

}

