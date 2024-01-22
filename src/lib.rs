use serde::{Deserialize, Serialize};

pub mod db;
pub use db::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub email: String,
    pub name: String,
    pub code: String,
    pub verified: bool,
    pub date: String,
}
