use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use std::fs::read_to_string;

pub mod db;

pub mod notifications;
pub use notifications::*;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct User {
    pub id: Thing,
    pub uid: usize,
    pub email: String,
    pub password: String,
    pub name: String,
    pub code: String,
    pub process: String,
    pub verified: bool,
    pub registration_date: DateTime<Utc>,
    pub verification_date: Option<DateTime<Utc>>,
    pub code_generated_date: Option<DateTime<Utc>>,
    pub github: Option<String>,
    pub gitlab: Option<String>,
    pub linkedin: Option<String>,
    pub about: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Group {
    pub gid: usize,
    pub name: String,
    pub location: String,
    pub description: String,
    pub owner: usize,
    pub creation_date: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Membership {
    pub gid: usize,
    pub uid: usize,
    pub join_date: DateTime<Utc>,
    pub admin: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RSVP {
    pub eid: usize,
    pub uid: usize,
    pub date: DateTime<Utc>,
    pub status: bool,
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum EventStatus {
    Draft,
    Published,
    Cancelled,
    Hidden,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Event {
    pub eid: usize,
    pub title: String,
    pub date: DateTime<Utc>,
    pub location: String,
    pub group_id: usize,
    pub description: String,
    pub status: EventStatus,
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum AuditType {
    GroupCreated,
    JoinGroup,
    LeaveGroup,
    RSVPYes,
    RSVPYesAgain,
    RSVPNo,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Audit {
    pub date: DateTime<Utc>,
    pub atype: AuditType,
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

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug)]
pub enum EmailMethod {
    Sendgrid,
    Folder,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MyConfig {
    pub base_url: String,

    pub email: EmailMethod,

    pub sendgrid_api_key: Option<String>,
    pub email_folder: Option<String>,

    pub admins: Vec<String>,

    pub from_name: String,
    pub from_email: String,

    pub database_username: String,
    pub database_password: String,
    pub database_namespace: String,
    pub database_name: String,
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
