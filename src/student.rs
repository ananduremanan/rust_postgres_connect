use crate::gbs_db_connect;
use crate::utils::generic_db_connect::generic_db_connect;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tracing::debug;

#[derive(Serialize, Deserialize)]
struct Student {
    student_id: Option<i32>,
    first_name: Option<String>,
    last_name: Option<String>,
    grade: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct SetStudentParams {
    mode: i32,
    #[serde(flatten)]
    student: Student,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteStudentParams {
    mode: i32,
    student_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseResponse {
    status: String,
    message: String,
}

// http GET function with calling a postgres function
pub async fn get_students(
    State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "get_student_names".to_string();
    let params = json!({"mode": 1});

    gbs_db_connect::<Student>(State(pg_pool), function_name, params).await
}

pub async fn set_students(
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

    generic_db_connect::<DatabaseResponse>(State(pg_pool), function_name, params).await
}

pub async fn delete_student(
    State(pg_pool): State<PgPool>,
    Json(params): Json<DeleteStudentParams>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "set_student".to_string();
    let params = json!({
    "mode": params.mode,
    "student_id": params.student_id
    });

    generic_db_connect::<DatabaseResponse>(State(pg_pool), function_name, params).await
}

pub async fn update_student(
    State(pg_pool): State<PgPool>,
    Json(params): Json<SetStudentParams>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "set_student".to_string();
    let student_params = json!({
        "mode": params.mode,
        "student": {
            "student_id": params.student.student_id,
            "first_name": params.student.first_name,
            "last_name": params.student.last_name,
            "grade": params.student.grade,
        }
    });

    debug!("Constructed student_params: {:?}", student_params);

    generic_db_connect::<DatabaseResponse>(State(pg_pool), function_name, student_params).await
}

pub async fn mock_costly_operation(
    State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let function_name = "get_student_names".to_string();
    let params = json!({"mode": 2});

    generic_db_connect::<Student>(State(pg_pool), function_name, params).await
}
