//! Simulate domain events for visualization testing
//!
//! This example publishes simulated domain events to NATS
//! to test the event visualization system.

use async_nats::Client;
use serde_json::json;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS
    let client = Client::connect("nats://localhost:4222").await?;
    println!("Connected to NATS server, simulating domain events...");

    // Domain configurations
    let domains = vec![
        ("graph", vec!["NodeCreated", "EdgeCreated", "NodeUpdated", "NodeDeleted"]),
        ("agent", vec!["AgentCreated", "TaskAssigned", "TaskCompleted", "AgentUpdated"]),
        ("workflow", vec!["WorkflowStarted", "StepExecuted", "WorkflowCompleted", "WorkflowFailed"]),
        ("person", vec!["PersonCreated", "PersonUpdated", "ContactAdded", "ProfileUpdated"]),
        ("organization", vec!["OrganizationCreated", "MemberAdded", "MemberRemoved", "StatusChanged"]),
        ("document", vec!["DocumentCreated", "DocumentUpdated", "VersionCreated", "DocumentArchived"]),
    ];

    let mut rng = rand::thread_rng();
    let mut correlation_chains: Vec<String> = vec![];

    loop {
        // Select random domain and event type
        let domain_idx = rng.gen_range(0..domains.len());
        let (domain, events) = &domains[domain_idx];
        let event_type = events[rng.gen_range(0..events.len())];
        
        // Generate event ID and correlation
        let event_id = Uuid::new_v4().to_string();
        let aggregate_id = Uuid::new_v4().to_string();
        
        // 30% chance to start new correlation chain
        let correlation_id = if rng.gen_bool(0.3) || correlation_chains.is_empty() {
            let new_correlation = Uuid::new_v4().to_string();
            correlation_chains.push(new_correlation.clone());
            new_correlation
        } else {
            correlation_chains[rng.gen_range(0..correlation_chains.len())].clone()
        };
        
        // 50% chance to have causation from previous event
        let causation_id = if rng.gen_bool(0.5) && !correlation_chains.is_empty() {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };

        // Create event payload
        let event_payload = json!({
            "event_id": event_id,
            "aggregate_id": aggregate_id,
            "timestamp": Utc::now().to_rfc3339(),
            "correlation_id": correlation_id,
            "causation_id": causation_id,
            "version": 1,
            "data": generate_event_data(domain, event_type, &mut rng),
        });

        // Publish event
        let subject = format!("{}.{}.event.v1", domain, event_type.to_lowercase());
        client.publish(subject.clone(), serde_json::to_vec(&event_payload)?).await?;
        
        println!("Published {} event to {}", event_type, subject);
        
        // Keep only recent correlation chains
        if correlation_chains.len() > 10 {
            correlation_chains.remove(0);
        }
        
        // Random delay between events
        let delay_ms = rng.gen_range(100..2000);
        sleep(Duration::from_millis(delay_ms)).await;
    }
}

fn generate_event_data(domain: &str, event_type: &str, rng: &mut impl Rng) -> serde_json::Value {
    match (domain, event_type) {
        ("graph", "NodeCreated") => json!({
            "node_type": ["Component", "Module", "Service"][rng.gen_range(0..3)],
            "label": format!("Node-{}", rng.gen_range(1000..9999)),
            "metadata": {
                "importance": rng.gen_range(1..10),
                "category": ["core", "auxiliary", "external"][rng.gen_range(0..3)],
            }
        }),
        
        ("agent", "TaskAssigned") => json!({
            "task_id": Uuid::new_v4().to_string(),
            "agent_name": format!("Agent-{}", rng.gen_range(1..20)),
            "priority": ["high", "medium", "low"][rng.gen_range(0..3)],
            "estimated_duration": rng.gen_range(60..3600),
        }),
        
        ("workflow", "StepExecuted") => json!({
            "workflow_id": Uuid::new_v4().to_string(),
            "step_name": format!("Step-{}", rng.gen_range(1..10)),
            "status": ["success", "pending", "failed"][rng.gen_range(0..3)],
            "duration_ms": rng.gen_range(100..5000),
        }),
        
        ("person", "PersonCreated") => json!({
            "full_name": format!("User {}", rng.gen_range(1000..9999)),
            "email": format!("user{}@example.com", rng.gen_range(1000..9999)),
            "department": ["Engineering", "Sales", "Marketing", "Operations"][rng.gen_range(0..4)],
        }),
        
        ("organization", "MemberAdded") => json!({
            "person_id": Uuid::new_v4().to_string(),
            "role": ["Developer", "Manager", "Director", "Analyst"][rng.gen_range(0..4)],
            "start_date": Utc::now().to_rfc3339(),
        }),
        
        _ => json!({
            "generic_field": format!("Value-{}", rng.gen_range(1..100)),
            "timestamp": Utc::now().to_rfc3339(),
        })
    }
}