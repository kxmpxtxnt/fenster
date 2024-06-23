use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BackendConfig {
    pub host: Host,
    pub postgres: Postgres,
    pub redis: Host,
    pub school: School
}

#[derive(Serialize, Deserialize)]
pub struct Postgres {
    pub address: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Host {
    pub address: String,
    pub port: u16
}

#[derive(Serialize, Deserialize)]
pub struct School {
    pub name: String,
    pub mail_pattern: String,
}