//! Core WildFly domain model.
//!
//! Defines the server types (standalone, domain controller, host controller),
//! admin container metadata, container instance representations, server
//! definitions for managed domains, and the management client configuration.

mod admin_image;
mod management;
mod server;
mod server_type;
mod start_spec;

/// Configuration for a named container instance with its admin container metadata.
pub trait ContainerConfig: Clone {
    fn admin_image(&self) -> &AdminImage;
    fn name(&self) -> &str;
}

macro_rules! impl_container_instance {
    ($type:ty) => {
        impl $crate::wildfly::ContainerConfig for $type {
            fn admin_image(&self) -> &$crate::wildfly::AdminImage {
                &self.admin_image
            }
            fn name(&self) -> &str {
                &self.name
            }
        }
    };
}

mod instance;

pub use admin_image::*;
pub use instance::*;
pub use management::*;
pub use server::*;
pub use server_type::*;
pub use start_spec::*;
