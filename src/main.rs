use tracing::{error, info};
use my_file_cloud_server::{storage, webserver, database, env};
use my_file_cloud_server::log::setup_logging;
use my_file_cloud_server::webserver::app_state::AppState;

#[tokio::main]
async fn main() {
    let config = env::load_config();
    
    let _guard = setup_logging(&config.log_directory);
    
    info!("Starting my-file-cloud-server");
    
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("Application panicked: {:?}", panic_info);
        
        default_hook(panic_info);
    }));
    
    let storage = match storage::Storage::new(config.cloud_root_directory) {
        Err(err) => panic!("Could not initialize cloud storage: {err}"),
        Ok(storage) => storage,
    };
    
    let database = database::setup_database(&config.database_config).await.expect("Failed to create Database");
    let app_state = AppState {
        jwt_secret: config.jwt_secret.as_bytes().to_vec(), 
        storage,
        database,
    };
    
    match webserver::start(config.webserver_port, app_state).await {
        Err(err) => panic!("Failed to start webserver: {err}"),
        _ => {},
    };
}
