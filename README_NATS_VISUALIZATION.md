# NATS Event Visualization

This module provides real-time 3D visualization of domain events flowing through NATS in the Alchemist system.

## Features

- **Real-time Event Streaming**: Subscribes to all domain events via NATS
- **3D Visualization**: Events appear as colored spheres in 3D space
- **Causation Chains**: Visual connections show event relationships
- **Force-Directed Layout**: Events self-organize based on relationships
- **Interactive Controls**: Navigate and interact with the event graph
- **Domain Color Coding**: Each domain has a unique color for easy identification
- **Event Retention**: Configurable time window for event display

## Architecture

The visualization system consists of:

1. **NATS Subscriber**: Listens to domain events on pattern `*.*.event.v1`
2. **Event Store**: Maintains recent events with configurable retention
3. **Event Flow Graph**: Tracks causation and correlation relationships
4. **Bevy ECS Systems**: Render events and update positions in real-time
5. **Force-Directed Layout**: Positions events based on relationships

## Usage

### Prerequisites

1. NATS server running (default: localhost:4222)
   
   On NixOS, add to your configuration.nix:
   ```nix
   services.nats = {
     enable = true;
     port = 4222;
   };
   ```
   
   Or use a nixos-container:
   ```nix
   containers.nats = {
     config = { config, pkgs, ... }: {
       services.nats = {
         enable = true;
         port = 4222;
       };
     };
   };
   ```

2. Build the project
   ```bash
   nix build
   # or
   cargo build --release
   ```

### Running the Visualization

1. Start the event visualization:
   ```bash
   cargo run --example nats_event_visualization_demo
   ```

2. In another terminal, simulate events:
   ```bash
   cargo run --example simulate_domain_events
   ```

### Controls

- **WASD/QE**: Move camera
- **Right Mouse + Drag**: Rotate camera view
- **Mouse Wheel**: Zoom in/out
- **P**: Pause/Resume event processing
- **C**: Clear all filters
- **1-3**: Filter by domain (graph, agent, workflow)
- **Left Click**: Select event for details

## Integration

To add NATS event visualization to your Bevy app:

```rust
use bevy::prelude::*;
use cim_domain_bevy::NatsEventVisualizationPlugin;
use async_nats::Client;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let nats_client = Arc::new(
        Client::connect("nats://localhost:4222").await.unwrap()
    );

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NatsEventVisualizationPlugin {
            nats_client,
            max_events: 200,
            retention_seconds: 600,
        })
        .run();
}
```

## Configuration

The `NatsEventVisualizationPlugin` accepts:
- `nats_client`: Connected NATS client
- `max_events`: Maximum events to display (default: 100)
- `retention_seconds`: How long to keep events (default: 300)

## Event Format

Events should be published to NATS with subject pattern:
```
{domain}.{aggregate_type}.event.v1
```

Event payload structure:
```json
{
  "event_id": "uuid",
  "aggregate_id": "uuid",
  "timestamp": "RFC3339",
  "correlation_id": "uuid",
  "causation_id": "uuid (optional)",
  "version": 1,
  "data": {
    // Domain-specific event data
  }
}
```

## Domain Colors

Each domain has a predefined color:
- Graph: Light Blue
- Agent: Red
- Workflow: Green
- Document: Yellow
- Person: Magenta
- Organization: Cyan
- Location: Purple
- Dialog: Orange
- Policy: Light Green
- Git: Brown
- Nix: Steel Blue
- Conceptual Spaces: Pink
- Identity: Yellow-Green

## Performance Considerations

- Events are processed in batches (10 per frame)
- Force-directed layout updates continuously
- Old events are automatically cleaned up
- Connections are recreated each frame (can be optimized)

## Future Enhancements

1. **Event Filtering UI**: GUI for filtering events
2. **Event Details Panel**: Show full event data on selection
3. **Correlation Highlighting**: Highlight all events in a correlation
4. **Time Scrubbing**: Replay events from history
5. **Statistics Dashboard**: Show event rates and patterns
6. **3D Clustering**: Group related events spatially
7. **Export/Import**: Save and load event sessions