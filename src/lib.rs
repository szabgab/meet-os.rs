use serde::{Deserialize, Serialize};

pub mod db;
pub use db::*;

pub mod notifications;
pub use notifications::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub email: String,
    pub password: String,
    pub name: String,
    pub code: String,
    pub process: String,
    pub verified: bool,
    pub date: String,
}

// TODO is there a better way to set the id of the event to the filename?
#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    #[serde(default = "get_empty_string")]
    pub id: String,
    pub title: String,
    pub date: String,
    pub location: String,
    pub group_id: usize,
    pub body: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Group {
    #[serde(default = "get_usize_zero")]
    pub gid: usize,
    pub name: String,
    pub location: String,
    pub description: String,
}

#[derive(Debug)]
pub struct EmailAddress {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Counter {
    name: String,
    count: u32,
}

fn get_empty_string() -> String {
    String::new()
}

fn get_usize_zero() -> usize {
    0
}
