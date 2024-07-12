use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use gbs_db_connect::gbs_db_connect;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Expose env variables
    dotenvy::dotenv().expect("Unable to Find .env File");

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
        .route("/get_student_names", get(get_students))
        .route("/set_student_names", post(set_students))
        .route("/delete_student", post(delete_student))
        .route("/update_student", post(update_student))
        .with_state(pg_pool)
        .layer(cors);

    //serve appliction
    axum::serve(listener, app).await.expect("Error Serving App");
}

#[derive(Serialize, Deserialize)]
struct Student {
    student_id: Option<i32>,
    first_name: Option<String>,
    last_name: Option<String>,
    grade: Option<i32>,
}

// http GET function with calling a postgres function
async fn get_students(
    State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "get_student_names".to_string();
    let params = json!({"mode": 1});

    gbs_db_connect::<Student>(State(pg_pool), function_name, params).await
}

#[derive(Serialize, Deserialize)]
struct SetStudentParams {
    mode: i32,
    #[serde(flatten)]
    student: Student,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseResponse {
    status: String,
    message: String,
}

async fn set_students(
    State(pg_pool): State<PgPool>,
    Json(params): Json<SetStudentParams>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "set_student".to_string();
    let params = json!({
        "mode": params.mode,
        "student": {
            "student_id": params.student.student_id,
            "first_name": params.student.first_name,
            "last_name": params.student.last_name,
            "grade": params.student.grade,
        }
    });

    gbs_db_connect::<DatabaseResponse>(State(pg_pool), function_name, params).await
}

#[derive(Serialize, Deserialize)]
struct DeleteStudentParams {
    mode: i32,
    student_id: Option<i32>,
}

async fn delete_student(
    State(pg_pool): State<PgPool>,
    Json(params): Json<DeleteStudentParams>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "set_student".to_string();
    let params = json!({
    "mode": params.mode,
    "student_id": params.student_id
    });

    gbs_db_connect::<DatabaseResponse>(State(pg_pool), function_name, params).await
}

async fn update_student(
    State(pg_pool): State<PgPool>,
    Json(params): Json<SetStudentParams>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "set_student".to_string();
    let params = json!({
        "mode": params.mode,
        "student": {
            "student_id": params.student.student_id,
            "first_name": params.student.first_name,
            "last_name": params.student.last_name,
            "grade": params.student.grade,
        }
    });

    gbs_db_connect::<DatabaseResponse>(State(pg_pool), function_name, params).await
}
