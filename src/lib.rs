//! CIM-Domain-Bevy: A Categorical Functor for Graph Visualization
//!
//! This library provides the isomorphic mapping between:
//! - **Domain Category**: CIM-ContextGraph (nodes, edges, contexts)
//! - **Visualization Category**: Bevy ECS (entities, components, systems)
//!
//! The functor preserves the categorical structure while enabling
//! high-performance visualization of domain graphs in Bevy applications.

pub mod bridge;
pub mod components;
pub mod events;
pub mod functors;
pub mod morphisms;
pub mod plugin;
pub mod resources;

// Re-export commonly used types
pub use components::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;

// Re-export bridge types selectively to avoid conflicts
pub use bridge::{AsyncSyncBridge, BridgeError};

// Re-export functor types
pub use functors::{DomainToVisualFunctor, VisualToDomainFunctor};
