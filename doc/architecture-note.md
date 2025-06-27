# Bevy Domain Architecture Note

## DDD Implementation Using ECS

The Bevy domain implements Domain-Driven Design (DDD) patterns using Entity Component System (ECS) constructs. This is not a replacement of DDD with ECS, but rather a demonstration of how DDD principles can be respected and implemented within an ECS framework.

## Key Mappings

### DDD to ECS Implementation

| DDD Concept          | ECS Implementation        | Example                                                    |
| -------------------- | ------------------------- | ---------------------------------------------------------- |
| **Aggregate**        | Entity + Component Bundle | `VisualNodeAggregate` (Entity with NodeId, Position, etc.) |
| **Value Objects**    | Components                | `Position`, `NodeId`, `NodeVisual`                         |
| **Commands**         | Events (Input)            | `CreateVisualNode`, `MoveNode`                             |
| **Command Handlers** | Systems                   | `handle_create_visual_node` system                         |
| **Domain Events**    | Events (Output)           | `VisualNodeCreated`, `NodeMoved`                           |
| **Queries**          | Query Systems             | `query_nodes_in_graph`, `find_nearest_node_system`         |
| **Projections**      | Resources                 | `GraphViewProjection`, `SpatialIndexProjection`            |

## Design Principles

### 1. Aggregates as Entities
- Aggregates are entities with specific component bundles
- Components define the aggregate's state
- Systems enforce business rules and invariants
- Example: `VisualNodeAggregate` is a bundle of components that together form the aggregate

### 2. Commands as Events
- Commands are Bevy events that trigger systems
- Systems act as command handlers
- Business logic lives in systems, not components
- Example: `CreateVisualNode` event → `handle_create_visual_node` system

### 3. Value Objects as Components
- Components are immutable value objects
- They hold domain data without behavior
- All behavior is in systems
- Example: `Position` component with validation in systems

### 4. Queries as Systems
- Query operations are implemented as systems or functions
- They read component data to answer domain questions
- Can be wrapped for domain-friendly interfaces
- Example: `query_selected_nodes` returns domain-relevant data

### 5. Projections as Resources
- Resources act as read models
- Updated by systems processing domain events
- Provide optimized views for queries
- Example: `GraphViewProjection` maintains graph structure

## Benefits

1. **Performance**: ECS provides cache-friendly data layout
2. **Parallelism**: Systems can run in parallel when they don't conflict
3. **Flexibility**: Component composition allows runtime flexibility
4. **Clarity**: Clear separation of data (components) and behavior (systems)
5. **DDD Compliance**: All DDD patterns are preserved and respected

## Example Flow

```
1. User clicks to create node
2. Input system sends CreateVisualNode command (event)
3. handle_create_visual_node system processes command
4. System creates entity with VisualNodeAggregate bundle
5. System emits VisualNodeCreated domain event
6. update_graph_projection system updates read model
7. Render system displays the new node
```

## Best Practices

1. **Keep Components Pure**: Components should only contain data, no logic
2. **Business Rules in Systems**: All domain logic belongs in systems
3. **Events for Communication**: Use events for all inter-system communication
4. **Respect Aggregates**: Maintain aggregate boundaries through component bundles
5. **Domain Language**: Use domain terms in component and system names

## Conclusion

The Bevy domain successfully implements DDD patterns using ECS constructs. This approach demonstrates that ECS and DDD are complementary, not competing paradigms. By mapping DDD concepts to ECS constructs, we maintain the benefits of both approaches while building a performant, maintainable visual layer for the CIM system. 