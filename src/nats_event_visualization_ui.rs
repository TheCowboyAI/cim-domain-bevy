//! NATS Event Visualization UI Components
//!
//! This module provides UI components for filtering and statistics display
//! for the NATS event visualization system.

use bevy::prelude::*;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::nats_event_visualization::{DomainEventReceived, EventStore};

/// Plugin for event visualization UI
pub struct EventVisualizationUIPlugin;

impl Plugin for EventVisualizationUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventFilters::default())
           .insert_resource(EventStatistics::default())
           .insert_resource(UIState::default())
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (
               update_statistics,
               handle_filter_input,
               update_filter_display,
               update_statistics_display,
           ).chain());
    }
}

/// Event filters for controlling what events are displayed
#[derive(Resource, Default, Debug, Clone)]
pub struct EventFilters {
    /// Filter by domain name
    pub domain_filter: Option<String>,
    /// Filter by event type
    pub event_type_filter: Option<String>,
    /// Filter by aggregate type
    pub aggregate_type_filter: Option<String>,
    /// Show only events with causation chains
    pub show_only_linked: bool,
    /// Time window filter (minutes)
    pub time_window_minutes: Option<u32>,
    /// Search filter for event content
    pub search_text: Option<String>,
}

impl EventFilters {
    /// Check if an event matches the current filters
    pub fn matches(&self, event: &DomainEventReceived) -> bool {
        // Domain filter
        if let Some(domain) = &self.domain_filter {
            if !event.domain.contains(domain) {
                return false;
            }
        }

        // Event type filter
        if let Some(event_type) = &self.event_type_filter {
            if !event.event_type.contains(event_type) {
                return false;
            }
        }

        // Aggregate type filter
        if let Some(agg_type) = &self.aggregate_type_filter {
            if !event.aggregate_type.contains(agg_type) {
                return false;
            }
        }

        // Linked events filter
        if self.show_only_linked && event.causation_id.is_none() && event.correlation_id.is_none() {
            return false;
        }

        // Time window filter
        if let Some(minutes) = self.time_window_minutes {
            let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);
            if event.timestamp < cutoff {
                return false;
            }
        }

        // Search text filter
        if let Some(search) = &self.search_text {
            let search_lower = search.to_lowercase();
            let event_text = format!(
                "{} {} {} {}", 
                event.domain, 
                event.event_type, 
                event.aggregate_type,
                event.payload.to_string()
            ).to_lowercase();
            
            if !event_text.contains(&search_lower) {
                return false;
            }
        }

        true
    }
}

/// Statistics about the event stream
#[derive(Resource, Default, Debug)]
pub struct EventStatistics {
    /// Total events received
    pub total_events: u64,
    /// Events per domain
    pub events_by_domain: HashMap<String, u64>,
    /// Events per type
    pub events_by_type: HashMap<String, u64>,
    /// Average events per second over the last minute
    pub events_per_second: f32,
    /// Number of causation chains
    pub causation_chains: u32,
    /// Number of correlation groups
    pub correlation_groups: u32,
    /// Busiest domain
    pub busiest_domain: Option<(String, u64)>,
    /// Most common event type
    pub most_common_event: Option<(String, u64)>,
    /// Last update time
    pub last_update: DateTime<Utc>,
}

/// UI state management
#[derive(Resource, Default)]
struct UIState {
    /// Whether the filter panel is expanded
    filter_panel_expanded: bool,
    /// Whether the statistics panel is expanded
    stats_panel_expanded: bool,
    /// Current domain filter input
    domain_input: String,
    /// Current event type filter input
    event_type_input: String,
    /// Current search input
    search_input: String,
}

/// Marker component for the filter UI panel
#[derive(Component)]
struct FilterPanel;

/// Marker component for the statistics UI panel
#[derive(Component)]
struct StatsPanel;

/// Marker component for filter input fields
#[derive(Component)]
struct FilterInput {
    filter_type: FilterType,
}

#[derive(Debug, Clone, Copy)]
enum FilterType {
    Domain,
    EventType,
    Search,
}

/// Setup the UI components
fn setup_ui(mut commands: Commands) {
    // Root UI node
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
    )).with_children(|parent| {
        // Left panel - Filters
        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            FilterPanel,
        )).with_children(|parent| {
            // Filter header
            parent.spawn((
                Text::new("Filters"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Domain filter
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    Text::new("Domain:"),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    FilterInput { filter_type: FilterType::Domain },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(""),
                        TextColor(Color::WHITE),
                        FilterInputText(FilterType::Domain),
                    ));
                });
            });
            
            // Event type filter
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    Text::new("Event Type:"),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    FilterInput { filter_type: FilterType::EventType },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(""),
                        TextColor(Color::WHITE),
                        FilterInputText(FilterType::EventType),
                    ));
                });
            });
            
            // Search filter
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    Text::new("Search:"),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    FilterInput { filter_type: FilterType::Search },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(""),
                        TextColor(Color::WHITE),
                        FilterInputText(FilterType::Search),
                    ));
                });
            });

            // Toggle filters
            parent.spawn((
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("Show Only Linked Events"),
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });

        // Right panel - Statistics
        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            StatsPanel,
        )).with_children(|parent| {
            // Stats header
            parent.spawn((
                Text::new("Statistics"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Stats content (will be populated dynamically)
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                StatisticsDisplay,
            ));
        });
    });
}


/// Marker component for filter input text
#[derive(Component)]
struct FilterInputText(FilterType);

/// Marker component for statistics display
#[derive(Component)]
struct StatisticsDisplay;

/// Update statistics based on event store
fn update_statistics(
    event_store: Res<EventStore>,
    mut statistics: ResMut<EventStatistics>,
    _time: Res<Time>,
) {
    // Only update every second to avoid performance impact
    let now = Utc::now();
    if (now - statistics.last_update).num_milliseconds() < 1000 {
        return;
    }

    statistics.last_update = now;

    // Calculate statistics from event store
    let events = event_store.get_recent_events(300); // Last 5 minutes
    
    statistics.total_events = events.len() as u64;
    
    // Reset counters
    statistics.events_by_domain.clear();
    statistics.events_by_type.clear();
    
    let mut causation_ids = std::collections::HashSet::new();
    let mut correlation_ids = std::collections::HashSet::new();
    
    for event in events {
        // Count by domain
        *statistics.events_by_domain.entry(event.domain.clone()).or_insert(0) += 1;
        
        // Count by type
        *statistics.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        
        // Track causation chains
        if let Some(causation_id) = &event.causation_id {
            causation_ids.insert(causation_id.clone());
        }
        
        // Track correlation groups
        if let Some(correlation_id) = &event.correlation_id {
            correlation_ids.insert(correlation_id.clone());
        }
    }
    
    statistics.causation_chains = causation_ids.len() as u32;
    statistics.correlation_groups = correlation_ids.len() as u32;
    
    // Find busiest domain
    statistics.busiest_domain = statistics.events_by_domain.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(domain, count)| (domain.clone(), *count));
    
    // Find most common event
    statistics.most_common_event = statistics.events_by_type.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(event_type, count)| (event_type.clone(), *count));
    
    // Calculate events per second (over the last minute)
    let one_minute_ago = now - chrono::Duration::seconds(60);
    let recent_count = events.iter()
        .filter(|e| e.timestamp > one_minute_ago)
        .count();
    statistics.events_per_second = recent_count as f32 / 60.0;
}

/// Handle filter input changes
fn handle_filter_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UIState>,
    mut filters: ResMut<EventFilters>,
    mut filter_texts: Query<(&mut Text, &FilterInputText)>,
) {
    // Handle keyboard input for active filter field
    // This is a simplified version - in production you'd want proper text input handling
    
    if keyboard.just_pressed(KeyCode::Enter) {
        // Apply current input values to filters
        filters.domain_filter = if ui_state.domain_input.is_empty() { 
            None 
        } else { 
            Some(ui_state.domain_input.clone()) 
        };
        
        filters.event_type_filter = if ui_state.event_type_input.is_empty() { 
            None 
        } else { 
            Some(ui_state.event_type_input.clone()) 
        };
        
        filters.search_text = if ui_state.search_input.is_empty() { 
            None 
        } else { 
            Some(ui_state.search_input.clone()) 
        };
    }
    
    // Update displayed text
    for (mut text, filter_type) in filter_texts.iter_mut() {
        match filter_type.0 {
            FilterType::Domain => text.0 = ui_state.domain_input.clone(),
            FilterType::EventType => text.0 = ui_state.event_type_input.clone(),
            FilterType::Search => text.0 = ui_state.search_input.clone(),
        }
    }
}

/// Update filter display to show active filters
fn update_filter_display(
    filters: Res<EventFilters>,
    mut filter_panel: Query<&mut BackgroundColor, With<FilterPanel>>,
) {
    // Change panel color if filters are active
    let has_filters = filters.domain_filter.is_some() 
        || filters.event_type_filter.is_some()
        || filters.search_text.is_some()
        || filters.show_only_linked;
        
    if let Ok(mut bg_color) = filter_panel.get_single_mut() {
        if has_filters {
            bg_color.0 = Color::srgba(0.1, 0.15, 0.1, 0.9); // Greenish tint
        } else {
            bg_color.0 = Color::srgba(0.1, 0.1, 0.1, 0.9); // Default
        }
    }
}

/// Update statistics display
fn update_statistics_display(
    statistics: Res<EventStatistics>,
    mut stats_display: Query<&mut Text, With<StatisticsDisplay>>,
) {
    if let Ok(mut text) = stats_display.get_single_mut() {
        text.0 = format!(
            "Total Events: {}\n\
             Events/sec: {:.1}\n\
             Causation Chains: {}\n\
             Correlation Groups: {}\n\
             \n\
             Busiest Domain:\n  {} ({})\n\
             \n\
             Most Common Event:\n  {} ({})\n\
             \n\
             Domains: {}\n\
             Event Types: {}",
            statistics.total_events,
            statistics.events_per_second,
            statistics.causation_chains,
            statistics.correlation_groups,
            statistics.busiest_domain.as_ref().map(|(d, _)| d.as_str()).unwrap_or("N/A"),
            statistics.busiest_domain.as_ref().map(|(_, c)| c).unwrap_or(&0),
            statistics.most_common_event.as_ref().map(|(e, _)| e.as_str()).unwrap_or("N/A"),
            statistics.most_common_event.as_ref().map(|(_, c)| c).unwrap_or(&0),
            statistics.events_by_domain.len(),
            statistics.events_by_type.len(),
        );
    }
}