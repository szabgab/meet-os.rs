use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::fs::read_to_string;

pub mod db;

pub mod notifications;
pub use notifications::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub uid: usize,
    pub email: String,
    pub password: String,
    pub name: String,
    pub code: String,
    pub process: String,
    pub verified: bool,
    pub registration_date: DateTime<Utc>,
    pub verification_date: Option<DateTime<Utc>>,
    pub github: Option<String>,
    pub gitlab: Option<String>,
    pub linkedin: Option<String>,
    pub about: Option<String>,
}

// TODO is there a better way to set the id of the event to the filename?
#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub eid: usize,
    pub title: String,
    pub date: String,
    pub location: String,
    pub group_id: usize,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Group {
    pub gid: usize,
    pub name: String,
    pub location: String,
    pub description: String,
    pub owner: usize,
    pub creation_date: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Membership {
    pub gid: usize,
    pub uid: usize,
    pub admin: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Audit {
    pub date: DateTime<Utc>,
    pub text: String,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct PublicConfig {
    google_analytics: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MyConfig {
    pub base_url: String,

    #[serde(default = "get_empty_string")]
    pub sendgrid_api_key: String,

    pub admins: Vec<String>,
}

fn get_empty_string() -> String {
    String::new()
}

/// # Panics
///
/// Panics when it fails to read the config file.
#[must_use]
pub fn get_public_config() -> PublicConfig {
    let filename = "config.yaml";
    let raw_string = read_to_string(filename).unwrap();
    let data: PublicConfig = serde_yaml::from_str(&raw_string).expect("YAML parsing error");
    data
}
