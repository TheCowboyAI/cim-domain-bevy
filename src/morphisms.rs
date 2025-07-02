//! Morphisms: The arrows (mappings) between objects in our categories
//!
//! This module defines the morphisms (structure-preserving mappings) between:
//! - Objects in the CIM-ContextGraph category (nodes, edges, graphs)
//! - Objects in the Bevy ECS category (entities, components, systems)

use bevy::prelude::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use crate::events::*;
use std::collections::HashMap;

/// Resource for tracking node entity mappings
#[derive(Resource, Default)]
pub struct NodeEntityMap(HashMap<NodeId, Entity>);

impl NodeEntityMap {
    pub fn insert(&mut self, node_id: NodeId, entity: Entity) {
        self.0.insert(node_id, entity);
    }

    pub fn get(&self, node_id: &NodeId) -> Option<&Entity> {
        self.0.get(node_id)
    }
}

/// Morphism from domain node operations to visual node operations
pub trait NodeMorphism {
    /// Map domain node creation to visual entity spawn
    fn create_visual(&self, commands: &mut Commands, node_id: NodeId, graph_id: GraphId, position: Vec3) -> Entity;

    /// Map domain node deletion to visual entity despawn
    fn delete_visual(&self, commands: &mut Commands, entity: Entity);

    /// Map domain node update to visual component update
    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: NodeUpdate);

    /// Remove visual representation
    fn remove_visual(&self, entity: Entity, commands: &mut Commands);
}

/// Morphism from domain edge operations to visual edge operations
pub trait EdgeMorphism {
    /// Map domain edge creation to visual line/curve creation
    fn create_visual(&self, commands: &mut Commands, edge_id: EdgeId, source: Entity, target: Entity) -> Entity;

    /// Map domain edge deletion to visual removal
    fn delete_visual(&self, commands: &mut Commands, entity: Entity);

    /// Map domain edge update to visual update
    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: EdgeUpdate);
}

/// Morphism from visual interactions to domain events
pub trait InteractionMorphism {
    /// Map mouse click to domain selection event
    fn map_click(&self, world_pos: Vec3, entity: Entity) -> SelectionChanged;

    /// Map drag operation to domain position update
    fn map_drag(&self, entity: Entity, delta: Vec3) -> NodePositionChanged;

    /// Map keyboard input to domain command
    fn map_keyboard(&self, key: KeyCode, modifiers: Modifiers) -> Option<DomainCommand>;
}

/// Composition of morphisms
pub struct MorphismComposition;

impl MorphismComposition {
    /// Compose two morphisms: (g ∘ f)(x) = g(f(x))
    pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
    where
        F: Fn(A) -> B,
        G: Fn(B) -> C,
    {
        move |x| g(f(x))
    }
}

/// Identity morphism (preserves structure exactly)
pub struct IdentityMorphism;

impl IdentityMorphism {
    pub fn map<T>(x: T) -> T {
        x
    }
}

/// Isomorphism verification
pub struct IsomorphismVerifier;

impl IsomorphismVerifier {
    /// Verify that F ∘ G = Id and G ∘ F = Id
    pub fn verify_isomorphism<A, B, F, G>(f: F, g: G, a: A, b: B) -> bool
    where
        A: PartialEq + Clone,
        B: PartialEq + Clone,
        F: Fn(A) -> B,
        G: Fn(B) -> A,
    {
        let a_clone = a.clone();
        let b_clone = b.clone();

        // Check F ∘ G = Id_B
        let b_result = f(g(b_clone));
        let b_preserved = b_result == b;

        // Check G ∘ F = Id_A
        let a_result = g(f(a_clone));
        let a_preserved = a_result == a;

        a_preserved && b_preserved
    }
}

/// Concrete implementations

pub struct StandardNodeMorphism;

impl NodeMorphism for StandardNodeMorphism {
    fn create_visual(&self, commands: &mut Commands, node_id: NodeId, graph_id: GraphId, position: Vec3) -> Entity {
        commands.spawn(crate::components::NodeVisualBundle::new(node_id, graph_id, position)).id()
    }

    fn delete_visual(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).despawn();
    }

    fn update_visual(&self, commands: &mut Commands, entity: Entity, update: NodeUpdate) {
        match update {
            NodeUpdate::Position(pos) => {
                commands.entity(entity).insert(Transform::from_translation(pos));
            }
            NodeUpdate::Selected(selected) => {
                if selected {
                    commands.entity(entity).insert(crate::components::Selected);
                } else {
                    commands.entity(entity).remove::<crate::components::Selected>();
                }
            }
        }
    }

    fn remove_visual(&self, entity: Entity, commands: &mut Commands) {
        commands.entity(entity).despawn();
    }
}

/// Helper types for morphism parameters
#[derive(Debug, Clone)]
pub enum NodeUpdate {
    Position(Vec3),
    Selected(bool),
}

#[derive(Debug, Clone)]
pub enum EdgeUpdate {
    Highlighted(bool),
    Weight(f32),
}

#[derive(Debug, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone)]
pub enum DomainCommand {
    CreateNode { position: Vec3 },
    DeleteSelected,
    ConnectSelected,
    LayoutGraph,
}

/// System functions for morphism operations

/// System to create node visuals from events
pub fn create_node_visual(
    mut commands: Commands,
    mut events: EventReader<CreateNodeVisual>,
    mut visual_created: EventWriter<VisualNodeCreated>,
) {
    for event in events.read() {
        let entity = commands.spawn((
            crate::components::NodeVisualBundle::new(
                event.node_id,
                GraphId::new(), // TODO: Add graph_id to CreateNodeVisual event
                event.position,
            ),
        )).id();
        
        // Emit visual created event
        visual_created.write(VisualNodeCreated {
            entity,
            node_id: event.node_id,
            position: event.position,
        });
    }
}

/// System to remove node visuals from events
pub fn remove_node_visual(
    mut commands: Commands,
    mut events: EventReader<RemoveNodeVisual>,
    query: Query<(Entity, &crate::components::NodeVisual)>,
) {
    for event in events.read() {
        // Find entities with matching node ID
        for (entity, node_visual) in query.iter() {
            if node_visual.node_id == event.node_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System to create edge visuals from events
pub fn create_edge_visual(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeVisual>,
    nodes: Query<(Entity, &crate::components::NodeVisual)>,
    mut visual_created: EventWriter<VisualEdgeCreated>,
) {
    for event in events.read() {
        // Find source and target entities by node ID
        let mut source_entity = None;
        let mut target_entity = None;

        for (entity, node_visual) in nodes.iter() {
            if node_visual.node_id == event.source_node_id {
                source_entity = Some(entity);
            }
            if node_visual.node_id == event.target_node_id {
                target_entity = Some(entity);
            }
        }

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            let entity = commands.spawn((
                crate::components::EdgeVisualBundle::new(
                    event.edge_id,
                    GraphId::new(), // TODO: Add graph_id to CreateEdgeVisual event
                    source,
                    target,
                ),
            )).id();
            
            // Emit visual created event
            visual_created.write(VisualEdgeCreated {
                entity,
                edge_id: event.edge_id,
                source_entity: source,
                target_entity: target,
            });
        }
    }
}

/// System to remove edge visuals from events
pub fn remove_edge_visual(
    mut commands: Commands,
    mut events: EventReader<RemoveEdgeVisual>,
    query: Query<(Entity, &crate::components::EdgeVisual)>,
) {
    for event in events.read() {
        // Find entities with matching edge ID
        for (entity, edge_visual) in query.iter() {
            if edge_visual.edge_id == event.edge_id {
                commands.entity(entity).despawn();
            }
        }
    }
}


