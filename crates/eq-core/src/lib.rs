//! Shared primitives and infrastructure for Ritma (spec/02 §2, ARC-02).

pub mod actor;
pub mod app_state;
pub mod attention;
pub mod brand;
pub mod clock;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod errors;
pub mod fields;
pub mod http;
pub mod ids;
pub mod jobs;
pub mod limits;
pub mod model;
pub mod module;
pub mod money;
pub mod outbox;
pub mod policy;
pub mod ports;
pub mod tenant;
pub mod text;
