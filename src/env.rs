use std::env;
use std::path::PathBuf;
use crate::database::{DatabaseConfig, MySQLDatabaseConfig};

pub struct MyFileCloudServerConfig {
    pub cloud_root_directory: PathBuf,
    pub log_directory: String,
    pub webserver_port: u16,
    pub jwt_secret: String,
    pub database_config: DatabaseConfig,
    pub client_origin: String,
}

pub fn load_config() -> MyFileCloudServerConfig {
    dotenvy::dotenv().ok();

    let db_mode = env::var("DATABASE_MODE")
        .expect("DATABASE_MODE missing");

    let database_config = match db_mode.as_str() {
        "mysql" => DatabaseConfig::MySQL(MySQLDatabaseConfig {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL missing"),

            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap()
                .parse()
                .expect("DB_MAX_CONNECTIONS must be a number"),
        }),

        "mock" => DatabaseConfig::Mock,

        other => panic!("Unknown DATABASE_MODE: {}", other),
    };

    MyFileCloudServerConfig {
        cloud_root_directory: env::var("CLOUD_ROOT_DIRECTORY").unwrap().into(),
        log_directory: env::var("LOG_DIRECTORY").unwrap(),
        webserver_port: env::var("WEBSERVER_PORT").unwrap().parse().unwrap(),
        jwt_secret: env::var("JWT_SECRET").unwrap(),
        client_origin: env::var("CLIENT_ORIGIN").unwrap(),
        database_config,
    }
}
