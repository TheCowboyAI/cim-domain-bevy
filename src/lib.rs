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
// pub mod deployment_visualization; // Disabled: depends on non-existent cim-domain-graph
pub mod edge_systems;
pub mod events;
pub mod functors;
pub mod layout;
pub mod morphisms;
pub mod nats_component_bridge;
pub mod nats_event_visualization;
pub mod nats_event_filter_ui;
pub mod nats_event_visualization_ui;
pub mod plugin;
pub mod resources;
pub mod visualization;

// Re-export commonly used types
pub use components::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;

// Re-export bridge types selectively to avoid conflicts
pub use bridge::{AsyncSyncBridge, BridgeError};

// Re-export functor types
pub use functors::{DomainToVisualFunctor, VisualToDomainFunctor};

// Re-export NATS event visualization
pub use nats_event_visualization::{NatsEventVisualizationPlugin, DomainEventReceived, EventVisualizationCommand};
pub use nats_event_visualization_ui::{EventVisualizationUIPlugin, EventFilters, EventStatistics};
pub use nats_event_filter_ui::{NatsEventFilterUIPlugin, EventFilterState, TimeRange};

// Re-export NATS component bridge for isomorphic architecture
pub use nats_component_bridge::{
    NatsComponentBridge, NatsComponentPlugin, NatsSyncedEntity, PendingComponentUpdate,
    process_nats_component_events, apply_component_updates,
};
