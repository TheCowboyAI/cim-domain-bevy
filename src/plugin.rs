//! Bevy Plugin for CIM Graph Visualization
//!
//! This plugin provides the complete functor implementation for visualizing
//! CIM-ContextGraph structures in Bevy applications.

use bevy::prelude::*;
use crate::events::*;
use crate::resources::*;
use crate::bridge::AsyncSyncBridge;


/// The main plugin that adds all graph visualization functionality
pub struct CimVizPlugin {
    /// Size of the event channels between domain and visualization
    pub channel_size: usize,
}

impl Default for CimVizPlugin {
    fn default() -> Self {
        Self {
            channel_size: 1000,
        }
    }
}

impl Plugin for CimVizPlugin {
    fn build(&self, app: &mut App) {
        // Register events
        app.add_event::<CreateNodeVisual>()
            .add_event::<RemoveNodeVisual>()
            .add_event::<CreateEdgeVisual>()
            .add_event::<RemoveEdgeVisual>()
            .add_event::<VisualNodeCreated>()
            .add_event::<VisualEdgeCreated>()
            .add_event::<NodeClicked>()
            .add_event::<NodeHovered>()
            .add_event::<NodeUnhovered>()
            .add_event::<EdgeClicked>()
            .add_event::<NodeDragStart>()
            .add_event::<NodeDragging>()
            .add_event::<NodeDragEnd>()
            .add_event::<NodePositionChanged>()
            .add_event::<NodeMetadataChanged>()
            // Add any additional events that need to be registered
            .add_event::<VisualizationCommand>();

        // Add resources
        app.insert_resource(AsyncSyncBridge::new(self.channel_size))
            .insert_resource(ActiveGraph::default())
            .insert_resource(Selection::default())
            .insert_resource(VisualizationConfig::default())
            .insert_resource(GraphLayoutConfig::default())
            .insert_resource(CameraState::default())
            .insert_resource(PerformanceMetrics::default())
            .insert_resource(GraphBounds::default())
            .insert_resource(ThemeConfig::default())
            .insert_resource(SpatialIndex::default())
            .insert_resource(InteractionState::default());

        // Add bridge systems
        app.add_systems(
            Update,
            (
                crate::bridge::process_domain_events,
                crate::bridge::send_visualization_commands,
            ),
        );

        // Add morphism systems
        app.add_systems(
            Update,
            (
                crate::morphisms::create_node_visual,
                crate::morphisms::remove_node_visual,
                crate::morphisms::create_edge_visual,
                crate::morphisms::remove_edge_visual,
            ),
        );
        
        // Add layout systems
        app.insert_resource(crate::layout::GraphLayoutState::default())
            .add_event::<crate::layout::SetLayoutAlgorithm>()
            .add_systems(
                Update,
                (
                    crate::layout::update_layout_from_hints,
                    crate::layout::apply_layout_algorithm,
                    crate::layout::handle_layout_commands,
                ),
            );
            
        // Add edge state systems
        app.add_event::<crate::edge_systems::EdgeStateChanged>()
            .add_systems(
                Update,
                (
                    crate::edge_systems::update_edge_visualization,
                    crate::edge_systems::highlight_connected_edges,
                    crate::edge_systems::update_edge_weights,
                    crate::edge_systems::handle_edge_state_changes,
                    crate::edge_systems::animate_edge_flow,
                ),
            );


    }
}

/// Optional plugin for debug visualization
pub struct CimVizDebugPlugin;

impl Plugin for CimVizDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                debug_log_events,
                debug_show_metrics,
            )
        );
    }
}

/// Debug system to log all visualization events
fn debug_log_events(
    mut node_clicks: EventReader<NodeClicked>,
    mut creates: EventReader<CreateNodeVisual>,
    mut removes: EventReader<RemoveNodeVisual>,
) {
    for event in node_clicks.read() {
        debug!("Node clicked: {:?}", event.node_id);
    }

    for event in creates.read() {
        debug!("Creating node visual: {:?}", event.node_id);
    }

    for event in removes.read() {
        debug!("Removing node visual: {:?}", event.node_id);
    }
}

/// Debug system to show performance metrics
fn debug_show_metrics(
    metrics: Res<PerformanceMetrics>,
    time: Res<Time>,
) {
    if time.elapsed_secs() % 5.0 < time.delta_secs() {
        debug!("Graph Performance Metrics:");
        debug!("  Node count: {}", metrics.node_count);
        debug!("  Edge count: {}", metrics.edge_count);
        debug!("  Layout time: {:.2}ms", metrics.layout_time_ms);
        debug!("  Render time: {:.2}ms", metrics.render_time_ms);
    }
}
