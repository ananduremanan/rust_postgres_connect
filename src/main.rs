use axum::{
    routing::{get, post},
    Router,
};
use gbs_db_connect::gbs_db_connect;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod student;

#[tokio::main]
async fn main() {
    // Expose env variables
    dotenvy::dotenv().expect("Unable to Find .env File");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Cors middleware
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
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
        .expect("Can't COnnect!!");

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
        .with_state(pg_pool)
        .layer(cors);

    //serve appliction
    axum::serve(listener, app).await.expect("Error Serving App");
}
