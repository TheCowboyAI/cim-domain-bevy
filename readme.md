# CIM Bevy Domain

The Bevy domain provides Entity Component System (ECS) infrastructure and integration for the CIM system.

## Overview

The Bevy domain acts as the presentation and interaction layer, providing:
- ECS components for visual representation
- Systems for real-time updates
- Resource management
- Event integration with other domains
- UI and rendering capabilities

## Key Concepts

### Components
- **Position**: 3D spatial coordinates
- **Velocity**: Movement vectors
- **Health**: Entity vitality tracking
- **Visual**: Rendering properties

### Systems
- **Movement**: Updates positions based on velocity
- **Health**: Manages health changes and death
- **Rendering**: Visual representation updates
- **Input**: User interaction handling

### Resources
- **GameState**: Global application state
- **Configuration**: Runtime settings
- **AssetHandles**: Loaded assets

## Architecture

The Bevy domain follows ECS principles:
1. **Entities**: Unique identifiers
2. **Components**: Data containers
3. **Systems**: Logic processors
4. **Resources**: Shared state

## Integration

Bevy integrates with other domains through:
- Event bridges for domain events
- Component mapping from domain models
- System coordination with domain handlers

## Usage

See the `examples/` directory for usage patterns.
