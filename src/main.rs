use axum::{extract::State, http::StatusCode, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Expose env variables
    dotenvy::dotenv().expect("Unable to Find .env File");

    // Cors middleware
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods(Any);

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
        .with_state(pg_pool)
        .layer(cors);

    //serve appliction
    axum::serve(listener, app).await.expect("Error Serving App");
}

// #[derive(Serialize)]
// struct Student {
//     student_id: Option<i32>,
//     first_name: Option<String>,
//     last_name: Option<String>,
//     grade: Option<i32>,
// }

// async fn get_students(
//     State(pg_pool): State<PgPool>,
// ) -> Result<(StatusCode, String), (StatusCode, String)> {
//     let rows = sqlx::query_as!(
//         Student,
//         "SELECT student_id, first_name, last_name, grade FROM student"
//     )
//     .fetch_all(&pg_pool)
//     .await
//     .map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             json!({"success": false, "message": e.to_string()}).to_string(),
//         )
//     })?;

//     Ok((
//         StatusCode::OK,
//         json!({"success": true, "data": rows}).to_string(),
//     ))
// }

#[derive(Serialize, Deserialize)]
struct Student {
    student_id: Option<i32>,
    first_name: Option<String>,
    last_name: Option<String>,
    grade: Option<i32>,
}

async fn get_students(
    State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query("SELECT get_student_names() as students")
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    let students_json: Value = row.try_get("students")
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": "Failed to get JSON from row: ".to_string() + &e.to_string()}).to_string(),
            )
        })?;

    let students: Vec<Student> = serde_json::from_value(students_json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": "Failed to deserialize students: ".to_string() + &e.to_string()}).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        json!({"success": true, "data": students}).to_string(),
    ))
}
