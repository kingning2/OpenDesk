//! DAG Builder 与 Runtime 冒烟测试。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use ports::workflow_runtime::CheckpointStore;
use serde_json::json;
use std::sync::Arc;
use workflow_runtime::{
    register_builtin_executors, DagBuilder, EdgeSpec, ExecutorRegistry, InMemoryCheckpointStore,
    InMemoryEventBus, InstanceId, NodeId, NodeSpec, NodeType, RetryPolicy, SchedulerConfig,
    WorkflowContext, WorkflowDefinition, WorkflowId, WorkflowRuntimeFacade, WorkflowState,
};

fn linear_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        id: WorkflowId::new("wf-linear"),
        nodes: vec![
            NodeSpec {
                id: NodeId::new("start"),
                node_type: NodeType::Start,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
            NodeSpec {
                id: NodeId::new("end"),
                node_type: NodeType::End,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
        ],
        edges: vec![EdgeSpec {
            id: "e1".to_string(),
            source: NodeId::new("start"),
            target: NodeId::new("end"),
            branch: None,
        }],
        run_policy: Default::default(),
    }
}

#[test]
fn dag_rejects_cycle() {
    let definition = WorkflowDefinition {
        id: WorkflowId::new("wf-cycle"),
        nodes: vec![
            NodeSpec {
                id: NodeId::new("start"),
                node_type: NodeType::Start,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
            NodeSpec {
                id: NodeId::new("a"),
                node_type: NodeType::Delay,
                config: json!({"delay_ms": 0}),
                retry: RetryPolicy::default(),
            },
            NodeSpec {
                id: NodeId::new("end"),
                node_type: NodeType::End,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
        ],
        edges: vec![
            EdgeSpec {
                id: "e1".to_string(),
                source: NodeId::new("start"),
                target: NodeId::new("a"),
                branch: None,
            },
            EdgeSpec {
                id: "e2".to_string(),
                source: NodeId::new("a"),
                target: NodeId::new("end"),
                branch: None,
            },
            EdgeSpec {
                id: "e3".to_string(),
                source: NodeId::new("end"),
                target: NodeId::new("a"),
                branch: None,
            },
        ],
        run_policy: Default::default(),
    };
    let err = DagBuilder::build(&definition).expect_err("cycle");
    let message = err.to_string();
    assert!(message.contains("illegal") || message.contains("cycle"));
}

#[test]
fn dag_rejects_multiple_starts() {
    let definition = WorkflowDefinition {
        id: WorkflowId::new("wf-multi-start"),
        nodes: vec![
            NodeSpec {
                id: NodeId::new("s1"),
                node_type: NodeType::Start,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
            NodeSpec {
                id: NodeId::new("s2"),
                node_type: NodeType::Start,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
            NodeSpec {
                id: NodeId::new("end"),
                node_type: NodeType::End,
                config: json!({}),
                retry: RetryPolicy::default(),
            },
        ],
        edges: vec![
            EdgeSpec {
                id: "e1".to_string(),
                source: NodeId::new("s1"),
                target: NodeId::new("end"),
                branch: None,
            },
            EdgeSpec {
                id: "e2".to_string(),
                source: NodeId::new("s2"),
                target: NodeId::new("end"),
                branch: None,
            },
        ],
        run_policy: Default::default(),
    };
    let err = DagBuilder::build(&definition).expect_err("multi start");
    assert!(err.to_string().contains("exactly one Start"));
}

#[tokio::test]
async fn runtime_runs_linear_graph_to_completion() {
    let mut registry = ExecutorRegistry::new();
    register_builtin_executors(&mut registry).expect("register");
    let store = Arc::new(InMemoryCheckpointStore::new());
    let bus = Arc::new(InMemoryEventBus::new());
    let facade = WorkflowRuntimeFacade::new(
        registry,
        store.clone(),
        bus.clone(),
        SchedulerConfig::default(),
    );

    let instance_id = facade
        .start(linear_definition(), WorkflowContext::new())
        .await
        .expect("start");

    let state = facade
        .get_instance_state(&instance_id)
        .expect("get")
        .expect("state");
    assert_eq!(state, WorkflowState::Completed);

    let nodes = store.list_nodes(instance_id.as_str()).expect("nodes");
    assert!(nodes.iter().any(|n| n.state == "succeeded"));
}

#[test]
fn instance_id_generate_unique() {
    let a = InstanceId::generate();
    let b = InstanceId::generate();
    assert_ne!(a.as_str(), b.as_str());
}
