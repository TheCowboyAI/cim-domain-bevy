//! Simple Workflow Demo without Graphics
//!
//! This demo shows the workflow concepts using just console output,
//! without requiring the full Bevy graphics stack.

use bevy::prelude::*;
use cim_contextgraph::NodeId;
use std::collections::HashMap;

fn main() {
    println!("🔄 Simple Workflow Visualization Demo\n");
    
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(WorkflowState::default())
        .add_systems(Startup, setup_workflow)
        .add_systems(Update, (
            simulate_workflow_progress,
            print_workflow_status,
        ))
        .run();
}

#[derive(Resource, Default)]
struct WorkflowState {
    nodes: HashMap<NodeId, WorkflowNode>,
    edges: Vec<WorkflowEdge>,
    current_step: usize,
    completed_steps: Vec<NodeId>,
}

#[derive(Debug, Clone)]
struct WorkflowNode {
    id: NodeId,
    name: String,
    node_type: NodeType,
    status: NodeStatus,
}

#[derive(Debug, Clone)]
struct WorkflowEdge {
    from: NodeId,
    to: NodeId,
    label: String,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Start,
    Process,
    Decision,
    End,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeStatus {
    Pending,
    Active,
    Completed,
}

fn setup_workflow(mut workflow: ResMut<WorkflowState>) {
    println!("📋 Setting up Document Approval Workflow\n");

    // Create workflow nodes
    let start_id = NodeId::new();
    let submit_id = NodeId::new();
    let review_id = NodeId::new();
    let decision_id = NodeId::new();
    let revise_id = NodeId::new();
    let approve_id = NodeId::new();
    let reject_id = NodeId::new();
    let end_id = NodeId::new();

    // Add nodes
    workflow.nodes.insert(start_id, WorkflowNode {
        id: start_id,
        name: "Start".to_string(),
        node_type: NodeType::Start,
        status: NodeStatus::Completed,
    });

    workflow.nodes.insert(submit_id, WorkflowNode {
        id: submit_id,
        name: "Submit Document".to_string(),
        node_type: NodeType::Process,
        status: NodeStatus::Active,
    });

    workflow.nodes.insert(review_id, WorkflowNode {
        id: review_id,
        name: "Review Document".to_string(),
        node_type: NodeType::Process,
        status: NodeStatus::Pending,
    });

    workflow.nodes.insert(decision_id, WorkflowNode {
        id: decision_id,
        name: "Decision".to_string(),
        node_type: NodeType::Decision,
        status: NodeStatus::Pending,
    });

    workflow.nodes.insert(revise_id, WorkflowNode {
        id: revise_id,
        name: "Revise Document".to_string(),
        node_type: NodeType::Process,
        status: NodeStatus::Pending,
    });

    workflow.nodes.insert(approve_id, WorkflowNode {
        id: approve_id,
        name: "Approve".to_string(),
        node_type: NodeType::Process,
        status: NodeStatus::Pending,
    });

    workflow.nodes.insert(reject_id, WorkflowNode {
        id: reject_id,
        name: "Reject".to_string(),
        node_type: NodeType::Process,
        status: NodeStatus::Pending,
    });

    workflow.nodes.insert(end_id, WorkflowNode {
        id: end_id,
        name: "End".to_string(),
        node_type: NodeType::End,
        status: NodeStatus::Pending,
    });

    // Add edges
    workflow.edges.push(WorkflowEdge {
        from: start_id,
        to: submit_id,
        label: "Begin Process".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: submit_id,
        to: review_id,
        label: "Submit for Review".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: review_id,
        to: decision_id,
        label: "Review Complete".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: decision_id,
        to: approve_id,
        label: "Approved".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: decision_id,
        to: revise_id,
        label: "Needs Revision".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: decision_id,
        to: reject_id,
        label: "Rejected".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: revise_id,
        to: submit_id,
        label: "Resubmit".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: approve_id,
        to: end_id,
        label: "Complete".to_string(),
    });

    workflow.edges.push(WorkflowEdge {
        from: reject_id,
        to: end_id,
        label: "Complete".to_string(),
    });

    workflow.completed_steps.push(start_id);

    // Print initial workflow structure
    println!("Workflow Nodes:");
    for node in workflow.nodes.values() {
        println!("  • {} ({:?}) - Status: {:?}", node.name, node.node_type, node.status);
    }

    println!("\nWorkflow Edges:");
    for edge in &workflow.edges {
        if let (Some(from), Some(to)) = (workflow.nodes.get(&edge.from), workflow.nodes.get(&edge.to)) {
            println!("  → {} → {} ({})", from.name, to.name, edge.label);
        }
    }
    
    println!("\n✅ Workflow setup complete!\n");
}

fn simulate_workflow_progress(
    mut workflow: ResMut<WorkflowState>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize timer
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(3.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    
    if timer.just_finished() {
        // Progress the workflow
        let steps = vec![
            ("Submit Document", "Document submitted successfully"),
            ("Review Document", "Document reviewed by manager"),
            ("Decision", "Decision made: Approved"),
            ("Approve", "Document approved and filed"),
            ("End", "Workflow completed"),
        ];

        if workflow.current_step < steps.len() {
            let (step_name, message) = steps[workflow.current_step];
            
            // Find the node to activate and collect nodes to complete
            let mut node_to_activate = None;
            let mut nodes_to_complete = Vec::new();
            
            for node in workflow.nodes.values() {
                if node.name == step_name {
                    node_to_activate = Some(node.id);
                } else if node.status == NodeStatus::Active {
                    nodes_to_complete.push(node.id);
                }
            }
            
            // Activate the new node
            if let Some(node_id) = node_to_activate {
                if let Some(node) = workflow.nodes.get_mut(&node_id) {
                    node.status = NodeStatus::Active;
                    println!("▶️  Processing: {} - {}", step_name, message);
                }
            }
            
            // Complete the previous active nodes
            for node_id in nodes_to_complete {
                if let Some(node) = workflow.nodes.get_mut(&node_id) {
                    node.status = NodeStatus::Completed;
                    workflow.completed_steps.push(node_id);
                }
            }
            
            workflow.current_step += 1;
            
            if workflow.current_step >= steps.len() {
                println!("\n🎉 Workflow completed successfully!");
                std::process::exit(0);
            }
        }
    }
}

fn print_workflow_status(
    workflow: Res<WorkflowState>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    
    // Print status every 60 frames (approximately once per second)
    if *frame_count % 60 == 0 {
        let active_count = workflow.nodes.values().filter(|n| n.status == NodeStatus::Active).count();
        let completed_count = workflow.nodes.values().filter(|n| n.status == NodeStatus::Completed).count();
        let pending_count = workflow.nodes.values().filter(|n| n.status == NodeStatus::Pending).count();
        
        println!("\n📊 Workflow Status:");
        println!("  Active: {}", active_count);
        println!("  Completed: {}", completed_count);
        println!("  Pending: {}", pending_count);
        
        if active_count > 0 {
            println!("\n  Currently active steps:");
            for node in workflow.nodes.values() {
                if node.status == NodeStatus::Active {
                    println!("    • {}", node.name);
                }
            }
        }
    }
} 