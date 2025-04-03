//! Technology fingerprinting module
//!
//! This module provides functionality to detect and identify technologies
//! used by web applications, servers, and other assets.

pub mod service;
pub mod web;

use crate::results::DiscoveryResult;
use shared::types::ID;

#[async_trait::async_trait]
pub trait Fingerprinter {
    /// Fingerprint a target and return discovered technologies
    async fn fingerprint(&self, target: &str, asset_id: ID) -> DiscoveryResult;
}
