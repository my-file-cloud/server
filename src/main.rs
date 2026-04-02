use tracing::{error, info};
use my_file_cloud_server::{storage, webserver, database};
use my_file_cloud_server::database::{DatabaseConfig, MySQLDatabaseConfig};
use my_file_cloud_server::log::setup_logging;
use my_file_cloud_server::webserver::app_state::AppState;

struct MyFileCloudServerConfig {
    cloud_root_directory: &'static str,
    log_directory: &'static str,
    webserver_port: u16,
    jwt_secret: &'static str,
    database_config: DatabaseConfig,
}

const CONFIG: MyFileCloudServerConfig = MyFileCloudServerConfig{
    cloud_root_directory: "P:\\my-file-cloud\\.cloud-root",
    log_directory: "P:\\my-file-cloud\\logs",
    webserver_port: 3000,
    jwt_secret: "MYTOPSECRET",
    database_config: DatabaseConfig::MySQL(MySQLDatabaseConfig {
        database_url: "mysql://root:root@localhost:3306/my_file_cloud",
        max_connections: 5,
    }),
};

#[tokio::main]
async fn main() {
    let _guard = setup_logging(&CONFIG.log_directory);
    
    info!("Starting my-file-cloud-server");
    
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("Application panicked: {:?}", panic_info);
        
        default_hook(panic_info);
    }));
    
    let storage = match storage::Storage::new(&CONFIG.cloud_root_directory) {
        Err(err) => panic!("Could not initialize cloud storage: {err}"),
        Ok(storage) => storage,
    };
    
    let database = database::setup_database(&CONFIG.database_config).await.expect("Failed to create Database");
    let app_state = AppState {
        jwt_secret: CONFIG.jwt_secret.as_bytes().to_vec(), 
        storage,
        database,
    };
    
    match webserver::start(CONFIG.webserver_port, app_state).await {
        Err(err) => panic!("Failed to start webserver: {err}"),
        _ => {},
    };
}