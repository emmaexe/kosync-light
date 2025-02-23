use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Deserialize)]
pub struct UserCreateReq {
    pub password: String,
    pub username: String
}

#[derive(Serialize)]
pub struct UserCreateRes<'a> {
    pub username: &'a str
}

#[derive(Serialize)]
pub struct UserAuthRes<'a> {
    pub authorized: &'a str
}

#[derive(Serialize, Deserialize)]
pub struct ProgressPutReq {
    pub device_id: String,
    pub percentage: Number,
    pub document: String,
    pub progress: String,
    pub device: String
}

#[derive(Serialize)]
pub struct ProgressPutRes {
    pub timestamp: Number,
    pub document: String
}

#[derive(Serialize)]
pub struct HealthCheckRes<'a> {
    pub state: &'a str
}

#[derive(Serialize)]
pub struct ErrorRes<'a> {
    pub message: &'a str,
    pub code: i32
}
