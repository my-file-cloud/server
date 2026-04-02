use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_logging(directory: &str) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(directory, "log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer();

    let env_filet = EnvFilter::from_default_env()
        .add_directive("sqlx=warn".parse().expect("Failed to parse tracing_appender EnvFilter directive"))
        .add_directive("axum=info".parse().expect("Failed to parse tracing_appender EnvFilter directive"))
        .add_directive("my_file_cloud_server=trace".parse().expect("Failed to parse tracing_appender EnvFilter directive"));
    
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(env_filet)
        .init();

    guard
}