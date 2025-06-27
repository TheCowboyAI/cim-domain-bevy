//! Basic ECS Example
//!
//! This example demonstrates:
//! - Basic Bevy ECS setup
//! - Component creation
//! - System implementation
//! - Resource management

use bevy::prelude::*;
use cim_domain_bevy::{
    components::{Position, Velocity, Health},
    systems::{movement_system, health_system},
    resources::GameState,
};

fn main() {
    println!("=== CIM Bevy Domain Example ===\n");

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(GameState::default())
        .add_systems(Update, (movement_system, health_system))
        .run();
}
