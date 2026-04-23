//! # wado — WildFly admin containers
//!
//! A library for building, running, and managing WildFly application server containers
//! across multiple versions and operation modes (standalone, domain controller, host controller).
//!
//! Uses podman (with docker fallback) as the container runtime, and images are hosted
//! at `quay.io/wado`.
//!
//! # Modules
//!
//! - [`wildfly`] — Core domain model: server types, admin containers, instances, and management client.
//! - [`container`] — Container runtime interaction: commands, queries, and name/port resolution.
//! - [`constants`] — Container naming, labels, and environment variable names.
//! - [`label`] — OCI label helpers for filtering and formatting container metadata.

pub mod constants;
pub mod container;
pub mod label;
pub mod wildfly;

pub use wildfly::*;
