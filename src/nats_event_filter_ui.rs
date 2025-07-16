//! Enhanced NATS Event Filtering UI and Statistics
//!
//! This module provides a comprehensive UI for filtering and analyzing
//! NATS events in real-time, with statistics and advanced filtering options.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::collections::{HashMap, HashSet, VecDeque};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Plugin for NATS event filtering UI
pub struct NatsEventFilterUIPlugin;

impl Plugin for NatsEventFilterUIPlugin {
    fn build(&self, app: &mut App) {
        // Only add EguiPlugin if not already added
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        
        app.insert_resource(EventFilterState::default())
           .insert_resource(EventStatistics::default())
           .insert_resource(FilterPresets::default())
           .add_systems(Update, (
               update_event_statistics,
               render_filter_ui,
               render_statistics_panel,
               apply_filters,
           ));
    }
}

/// State for event filtering
#[derive(Resource, Default, Debug, Clone)]
pub struct EventFilterState {
    /// Filter by domain
    pub domain_filters: HashSet<String>,
    
    /// Filter by event type
    pub event_type_filters: HashSet<String>,
    
    /// Filter by aggregate type
    pub aggregate_type_filters: HashSet<String>,
    
    /// Time range filter
    pub time_range: TimeRange,
    
    /// Search query for event content
    pub search_query: String,
    
    /// Show only correlated events
    pub only_correlated: bool,
    
    /// Show only events with errors
    pub only_errors: bool,
    
    /// Minimum event rate threshold (events per second)
    pub min_event_rate: Option<f32>,
    
    /// Active filter preset
    pub active_preset: Option<String>,
}

/// Time range for filtering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    LastMinute,
    LastFiveMinutes,
    LastFifteenMinutes,
    LastHour,
    LastDay,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange::LastFiveMinutes
    }
}

impl TimeRange {
    fn to_duration(&self) -> Duration {
        match self {
            TimeRange::LastMinute => Duration::minutes(1),
            TimeRange::LastFiveMinutes => Duration::minutes(5),
            TimeRange::LastFifteenMinutes => Duration::minutes(15),
            TimeRange::LastHour => Duration::hours(1),
            TimeRange::LastDay => Duration::days(1),
            TimeRange::Custom { start, end } => *end - *start,
        }
    }
    
    fn is_in_range(&self, timestamp: DateTime<Utc>) -> bool {
        let now = Utc::now();
        match self {
            TimeRange::Custom { start, end } => timestamp >= *start && timestamp <= *end,
            _ => now - timestamp <= self.to_duration(),
        }
    }
}

/// Statistics about events
#[derive(Resource, Default)]
pub struct EventStatistics {
    /// Total events received
    pub total_events: u64,
    
    /// Events per domain
    pub events_by_domain: HashMap<String, u64>,
    
    /// Events per type
    pub events_by_type: HashMap<String, u64>,
    
    /// Events per aggregate type
    pub events_by_aggregate: HashMap<String, u64>,
    
    /// Event rate history (events per second over time)
    pub event_rate_history: VecDeque<(DateTime<Utc>, f32)>,
    
    /// Error count
    pub error_count: u64,
    
    /// Average event size
    pub avg_event_size: f32,
    
    /// Peak event rate
    pub peak_event_rate: f32,
    
    /// Correlation chains
    pub correlation_chains: HashMap<String, Vec<String>>,
    
    /// Last update time
    pub last_update: DateTime<Utc>,
}

impl EventStatistics {
    /// Update statistics with new event
    pub fn update(&mut self, event: &super::nats_event_visualization::DomainEventReceived) {
        self.total_events += 1;
        
        // Update domain count
        *self.events_by_domain.entry(event.domain.clone()).or_insert(0) += 1;
        
        // Update event type count
        *self.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        
        // Update aggregate type count
        *self.events_by_aggregate.entry(event.aggregate_type.clone()).or_insert(0) += 1;
        
        // Update correlation chains
        if let Some(correlation_id) = &event.correlation_id {
            self.correlation_chains
                .entry(correlation_id.clone())
                .or_insert_with(Vec::new)
                .push(event.event_id.clone());
        }
        
        // Update event size (approximate from JSON)
        let event_size = event.payload.to_string().len() as f32;
        self.avg_event_size = (self.avg_event_size * (self.total_events - 1) as f32 + event_size) 
                            / self.total_events as f32;
        
        self.last_update = Utc::now();
    }
    
    /// Calculate current event rate
    pub fn current_event_rate(&self) -> f32 {
        if let Some((last_time, last_rate)) = self.event_rate_history.back() {
            *last_rate
        } else {
            0.0
        }
    }
    
    /// Get top domains by event count
    pub fn top_domains(&self, n: usize) -> Vec<(String, u64)> {
        let mut domains: Vec<_> = self.events_by_domain.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        domains.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        domains.truncate(n);
        domains
    }
    
    /// Get top event types by count
    pub fn top_event_types(&self, n: usize) -> Vec<(String, u64)> {
        let mut types: Vec<_> = self.events_by_type.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        types.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        types.truncate(n);
        types
    }
}

/// Filter presets for common scenarios
#[derive(Resource, Default)]
pub struct FilterPresets {
    presets: HashMap<String, EventFilterState>,
}

impl FilterPresets {
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        
        // High traffic preset
        let mut high_traffic = EventFilterState::default();
        high_traffic.min_event_rate = Some(10.0);
        presets.insert("High Traffic".to_string(), high_traffic);
        
        // Errors only preset
        let mut errors_only = EventFilterState::default();
        errors_only.only_errors = true;
        presets.insert("Errors Only".to_string(), errors_only);
        
        // Correlated events preset
        let mut correlated = EventFilterState::default();
        correlated.only_correlated = true;
        presets.insert("Correlated Events".to_string(), correlated);
        
        // Recent activity preset
        let mut recent = EventFilterState::default();
        recent.time_range = TimeRange::LastMinute;
        presets.insert("Recent Activity".to_string(), recent);
        
        Self { presets }
    }
    
    pub fn get(&self, name: &str) -> Option<&EventFilterState> {
        self.presets.get(name)
    }
    
    pub fn list(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }
}

/// Update event statistics
fn update_event_statistics(
    mut events: EventReader<super::nats_event_visualization::DomainEventReceived>,
    mut stats: ResMut<EventStatistics>,
    time: Res<Time>,
) {
    let mut event_count = 0;
    
    for event in events.read() {
        stats.update(event);
        event_count += 1;
    }
    
    // Update event rate
    if event_count > 0 {
        let current_time = Utc::now();
        let rate = event_count as f32 / time.delta_secs();
        
        stats.event_rate_history.push_back((current_time, rate));
        
        // Keep only last 60 seconds of history
        while stats.event_rate_history.len() > 60 {
            stats.event_rate_history.pop_front();
        }
        
        // Update peak rate
        if rate > stats.peak_event_rate {
            stats.peak_event_rate = rate;
        }
    }
}

/// Render the filter UI
fn render_filter_ui(
    mut contexts: EguiContexts,
    mut filter_state: ResMut<EventFilterState>,
    presets: Res<FilterPresets>,
    stats: Res<EventStatistics>,
) {
    egui::Window::new("Event Filters")
        .default_pos(egui::pos2(10.0, 100.0))
        .show(contexts.ctx_mut(), |ui| {
            // Preset selector
            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.button("None").clicked() {
                    filter_state.active_preset = None;
                }
                for preset_name in presets.list() {
                    if ui.button(&preset_name).clicked() {
                        if let Some(preset) = presets.get(&preset_name) {
                            *filter_state = preset.clone();
                            filter_state.active_preset = Some(preset_name);
                        }
                    }
                }
            });
            
            ui.separator();
            
            // Domain filters
            ui.collapsing("Domain Filters", |ui| {
                for (domain, _) in stats.events_by_domain.iter() {
                    let mut selected = filter_state.domain_filters.contains(domain);
                    if ui.checkbox(&mut selected, domain).changed() {
                        if selected {
                            filter_state.domain_filters.insert(domain.clone());
                        } else {
                            filter_state.domain_filters.remove(domain);
                        }
                    }
                }
            });
            
            // Event type filters
            ui.collapsing("Event Type Filters", |ui| {
                for (event_type, _) in stats.events_by_type.iter() {
                    let mut selected = filter_state.event_type_filters.contains(event_type);
                    if ui.checkbox(&mut selected, event_type).changed() {
                        if selected {
                            filter_state.event_type_filters.insert(event_type.clone());
                        } else {
                            filter_state.event_type_filters.remove(event_type);
                        }
                    }
                }
            });
            
            // Time range
            ui.horizontal(|ui| {
                ui.label("Time Range:");
                ui.selectable_value(&mut filter_state.time_range, TimeRange::LastMinute, "1m");
                ui.selectable_value(&mut filter_state.time_range, TimeRange::LastFiveMinutes, "5m");
                ui.selectable_value(&mut filter_state.time_range, TimeRange::LastFifteenMinutes, "15m");
                ui.selectable_value(&mut filter_state.time_range, TimeRange::LastHour, "1h");
                ui.selectable_value(&mut filter_state.time_range, TimeRange::LastDay, "24h");
            });
            
            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut filter_state.search_query);
            });
            
            // Options
            ui.checkbox(&mut filter_state.only_correlated, "Only Correlated Events");
            ui.checkbox(&mut filter_state.only_errors, "Only Errors");
            
            // Event rate filter
            ui.horizontal(|ui| {
                ui.label("Min Event Rate:");
                let mut enabled = filter_state.min_event_rate.is_some();
                ui.checkbox(&mut enabled, "");
                if enabled {
                    let mut rate = filter_state.min_event_rate.unwrap_or(1.0);
                    ui.add(egui::Slider::new(&mut rate, 0.1..=100.0).suffix(" events/s"));
                    filter_state.min_event_rate = Some(rate);
                } else {
                    filter_state.min_event_rate = None;
                }
            });
        });
}

/// Render the statistics panel
fn render_statistics_panel(
    mut contexts: EguiContexts,
    stats: Res<EventStatistics>,
) {
    egui::Window::new("Event Statistics")
        .default_pos(egui::pos2(300.0, 100.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Overview");
            
            egui::Grid::new("stats_overview")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Total Events:");
                    ui.label(format!("{}", stats.total_events));
                    ui.end_row();
                    
                    ui.label("Current Rate:");
                    ui.label(format!("{:.1} events/s", stats.current_event_rate()));
                    ui.end_row();
                    
                    ui.label("Peak Rate:");
                    ui.label(format!("{:.1} events/s", stats.peak_event_rate));
                    ui.end_row();
                    
                    ui.label("Error Count:");
                    ui.label(format!("{}", stats.error_count));
                    ui.end_row();
                    
                    ui.label("Avg Event Size:");
                    ui.label(format!("{:.0} bytes", stats.avg_event_size));
                    ui.end_row();
                });
            
            ui.separator();
            
            // Top domains
            ui.heading("Top Domains");
            for (domain, count) in stats.top_domains(5) {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: ", domain));
                    ui.label(format!("{} events", count));
                    
                    // Show percentage
                    let percentage = (count as f32 / stats.total_events as f32) * 100.0;
                    ui.label(format!("({:.1}%)", percentage));
                });
            }
            
            ui.separator();
            
            // Top event types
            ui.heading("Top Event Types");
            for (event_type, count) in stats.top_event_types(5) {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: ", event_type));
                    ui.label(format!("{} events", count));
                });
            }
            
            ui.separator();
            
            // Event rate graph (simple text representation)
            ui.heading("Event Rate History");
            if !stats.event_rate_history.is_empty() {
                // Simple sparkline using Unicode blocks
                let max_rate = stats.event_rate_history.iter()
                    .map(|(_, rate)| *rate)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(1.0);
                
                let sparkline: String = stats.event_rate_history.iter()
                    .map(|(_, rate)| {
                        let normalized = rate / max_rate;
                        match (normalized * 8.0) as u8 {
                            0 => ' ',
                            1 => '▁',
                            2 => '▂',
                            3 => '▃',
                            4 => '▄',
                            5 => '▅',
                            6 => '▆',
                            7 => '▇',
                            _ => '█',
                        }
                    })
                    .collect();
                
                ui.monospace(&sparkline);
                ui.label(format!("Max: {:.1} events/s", max_rate));
            }
            
            ui.separator();
            
            // Correlation chains
            ui.heading("Active Correlation Chains");
            ui.label(format!("{} chains", stats.correlation_chains.len()));
            
            // Show largest chains
            let mut chains: Vec<_> = stats.correlation_chains.iter()
                .map(|(id, events)| (id, events.len()))
                .collect();
            chains.sort_by_key(|(_, len)| std::cmp::Reverse(*len));
            
            for (correlation_id, length) in chains.iter().take(3) {
                ui.label(format!("{}: {} events", &correlation_id[..8], length));
            }
        });
}

/// Apply filters to events
fn apply_filters(
    filter_state: Res<EventFilterState>,
    mut visibility_query: Query<(&super::nats_event_visualization::EventVisual, &mut Visibility)>,
) {
    for (event_visual, mut visibility) in visibility_query.iter_mut() {
        let mut should_show = true;
        
        // Apply domain filter
        if !filter_state.domain_filters.is_empty() {
            should_show &= filter_state.domain_filters.contains(&event_visual.domain);
        }
        
        // Apply event type filter
        if !filter_state.event_type_filters.is_empty() {
            should_show &= filter_state.event_type_filters.contains(&event_visual.event_type);
        }
        
        // Apply time range filter
        should_show &= filter_state.time_range.is_in_range(event_visual.timestamp);
        
        // Apply correlation filter
        if filter_state.only_correlated {
            should_show &= event_visual.correlation_id.is_some();
        }
        
        // TODO: Apply search query filter (would need access to full event data)
        
        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_range() {
        let now = Utc::now();
        let five_min_ago = now - Duration::minutes(5);
        let ten_min_ago = now - Duration::minutes(10);
        
        let range = TimeRange::LastFiveMinutes;
        assert!(range.is_in_range(five_min_ago));
        assert!(!range.is_in_range(ten_min_ago));
    }
    
    #[test]
    fn test_statistics_update() {
        let mut stats = EventStatistics::default();
        
        let event = super::super::nats_event_visualization::DomainEventReceived {
            event_id: "test123".to_string(),
            timestamp: Utc::now(),
            domain: "Sales".to_string(),
            event_type: "OrderPlaced".to_string(),
            aggregate_id: "order123".to_string(),
            aggregate_type: "Order".to_string(),
            correlation_id: Some("corr123".to_string()),
            causation_id: None,
            payload: serde_json::json!({"test": "data"}),
        };
        
        stats.update(&event);
        
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.events_by_domain.get("Sales"), Some(&1));
        assert_eq!(stats.events_by_type.get("OrderPlaced"), Some(&1));
    }
}