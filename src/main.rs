use crate::fileuploader::UploadState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;
use tracing_appender::rolling::daily;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, FmtSubscriber};

mod constants;
mod fileuploader;
mod student;
mod utils;

// Logger function to log server logs
fn logging() {
    let file_appender = daily("logs", "server.log");
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .finish()
        .with(
            fmt::Layer::new()
                .with_writer(file_appender)
                .with_ansi(false),
        );

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[derive(Clone)]
struct AppState {
    pg_pool: PgPool,
    upload_state: Arc<Mutex<UploadState>>,
}

#[tokio::main]
async fn main() {
    // Expose env variables
    dotenvy::dotenv().expect("Unable to Find .env File");
    logging();

    let allow_origin =
        std::env::var("ORIGIN_ADDRESS").unwrap_or("http://localhost:5173".to_owned());

    // Cors middleware
    let cors = CorsLayer::new()
        .allow_origin([allow_origin.parse().unwrap()])
        .allow_methods(Any)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    // Set variables from env
    let server_addr = std::env::var("SERVER_ADDRESSS").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABSE_URL is missing!!");

    // create db pool
    let pg_pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("Can't Connect!!");

    let upload_state = Arc::new(Mutex::new(UploadState::new()));

    let app_state = AppState {
        pg_pool,
        upload_state,
    };

    // tcp listener
    let listener = TcpListener::bind(server_addr)
        .await
        .expect("Could Not Create TCP Listener");

    println!("Listening on {}", listener.local_addr().unwrap());

    // compose routes
    let app: Router = Router::new()
        .route("/", get(|| async { "Hello Nithya" }))
        .route("/get_student_names", get(student::get_students))
        .route("/set_student_names", post(student::set_students))
        .route("/delete_student", post(student::delete_student))
        .route("/update_student", post(student::update_student))
        .route("/mock_operation", get(student::mock_costly_operation))
        .route("/student/:student_id", delete(student::delete_by_id))
        .route("/student_update/:student_id", put(student::update_by_put))
        .route("/upload", post(fileuploader::handle_upload))
        .with_state(app_state)
        .layer(cors);

    //serve appliction
    tracing::info!("Server started");
    axum::serve(listener, app).await.expect("Error Serving App");
}
