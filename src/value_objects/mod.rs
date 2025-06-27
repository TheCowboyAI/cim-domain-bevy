//! Bevy Domain Value Objects
//!
//! In ECS, value objects are components that hold immutable domain data.
//! These components are attached to entities to represent state.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Node identifier component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Edge identifier component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub Uuid);

impl EdgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Graph identifier component
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

impl GraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Position component
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

/// Node visual appearance
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct NodeVisual {
    pub color: Color,
    pub size: f32,
    pub shape: NodeShape,
}

/// Edge visual appearance
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct EdgeVisual {
    pub color: Color,
    pub width: f32,
    pub style: EdgeStyle,
}

/// Node shapes
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NodeShape {
    #[default]
    Circle,
    Square,
    Diamond,
    Hexagon,
}

/// Edge styles
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EdgeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Interaction state
#[derive(Component, Debug, Clone, Default)]
pub struct InteractionState {
    pub is_hovered: bool,
    pub is_selected: bool,
    pub is_dragging: bool,
}

/// Node metadata
#[derive(Component, Debug, Clone, Default)]
pub struct NodeMetadata {
    pub label: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Source node reference
#[derive(Component, Debug, Clone)]
pub struct SourceNode(pub Entity);

/// Target node reference
#[derive(Component, Debug, Clone)]
pub struct TargetNode(pub Entity);

/// Edge curve information
#[derive(Component, Debug, Clone, Default)]
pub struct EdgeCurve {
    pub control_point: Option<Vec2>,
    pub curvature: f32,
}

/// Canvas state
#[derive(Component, Debug, Clone, Default)]
pub struct CanvasState {
    pub offset: Vec2,
    pub zoom: f32,
}

/// Viewport settings
#[derive(Component, Debug, Clone, Default)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

/// Render settings
#[derive(Component, Debug, Clone, Default)]
pub struct RenderSettings {
    pub antialiasing: bool,
    pub show_grid: bool,
    pub grid_size: f32,
}

/// Visual style for nodes
#[derive(Debug, Clone, PartialEq)]
pub struct NodeVisualStyle {
    pub color: Color,
    pub size: f32,
    pub shape: NodeShape,
    pub border_color: Color,
    pub border_width: f32,
}

/// Visual style for edges
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeVisualStyle {
    pub color: Color,
    pub width: f32,
    pub style: EdgeStyle,
    pub arrow_size: f32,
}
