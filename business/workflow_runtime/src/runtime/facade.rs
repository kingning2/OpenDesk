//! WorkflowRuntimeFacade：start / pause / resume / cancel。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::context::WorkflowContext;
use crate::dag::DagBuilder;
use crate::definition::WorkflowDefinition;
use crate::error::WorkflowError;
use crate::event::{WorkflowEvent, WorkflowEventBus};
use crate::executor::ExecutorRegistry;
use crate::id::InstanceId;
use crate::persistence::CheckpointGateway;
use crate::recover::RecoveryService;
use crate::scheduler::{Scheduler, SchedulerConfig, SchedulerHandle};
use crate::state::{transition_workflow_state, NodeState, WorkflowState, WorkflowTransition};
use ports::workflow_runtime::CheckpointStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 运行时门面（非 God Object：委托 Scheduler / Recovery / Checkpoint）。
///
/// @author coisini
/// @created 2026-07-23
pub struct WorkflowRuntimeFacade {
    registry: Arc<ExecutorRegistry>,
    checkpoint: CheckpointGateway,
    event_bus: Arc<dyn WorkflowEventBus>,
    scheduler: Arc<Scheduler>,
    recovery: RecoveryService,
    active: Arc<Mutex<HashMap<String, Arc<Mutex<SchedulerHandle>>>>>,
}

impl WorkflowRuntimeFacade {
    /// 装配 Runtime。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn new(
        registry: ExecutorRegistry,
        store: Arc<dyn CheckpointStore>,
        event_bus: Arc<dyn WorkflowEventBus>,
        config: SchedulerConfig,
    ) -> Self {
        let registry = Arc::new(registry);
        let checkpoint = CheckpointGateway::new(store);
        let scheduler = Arc::new(Scheduler::new(
            Arc::clone(&registry),
            checkpoint.clone(),
            Arc::clone(&event_bus),
            config,
        ));
        let recovery = RecoveryService::new(checkpoint.clone(), Arc::clone(&scheduler));
        Self {
            registry,
            checkpoint,
            event_bus,
            scheduler,
            recovery,
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 启动工作流。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param definition - 图定义
    /// @param initial_context - 初始 Context
    /// @returns 实例 id
    pub async fn start(
        &self,
        definition: WorkflowDefinition,
        initial_context: WorkflowContext,
    ) -> Result<InstanceId, WorkflowError> {
        let graph = DagBuilder::build(&definition)?;
        let instance_id = InstanceId::generate();

        let mut node_states = HashMap::new();
        let mut initial_pairs = Vec::new();
        for node_id in graph.nodes.keys() {
            let state = match node_id == &graph.start_id {
                true => NodeState::Ready,
                false => NodeState::Pending,
            };
            node_states.insert(node_id.clone(), state);
            initial_pairs.push((node_id.clone(), state));
        }

        let specs: Vec<&crate::definition::NodeSpec> = definition.nodes.iter().collect();
        self.checkpoint.create_instance(
            &instance_id,
            &definition,
            WorkflowState::Running,
            &initial_context,
            &specs,
            &initial_pairs,
        )?;

        let handle = SchedulerHandle {
            instance_id: instance_id.clone(),
            definition,
            graph,
            workflow_state: WorkflowState::Running,
            node_states,
            attempts: HashMap::new(),
            retry_states: HashMap::new(),
            context: initial_context,
            cancel_requested: false,
            pause_requested: false,
        };

        self.event_bus.publish(WorkflowEvent::WorkflowStarted {
            instance_id: instance_id.clone(),
            state: WorkflowState::Running,
        })?;

        let _ = transition_workflow_state(WorkflowState::Pending, WorkflowTransition::Start);

        let scheduler = Arc::clone(&self.scheduler);
        let handle_arc = Arc::new(Mutex::new(handle));
        {
            let mut guard = self.active.lock().await;
            guard.insert(instance_id.as_str().to_string(), Arc::clone(&handle_arc));
        }

        {
            let mut locked = handle_arc.lock().await;
            scheduler.run_until_idle(&mut locked).await?;
        }

        {
            let mut guard = self.active.lock().await;
            guard.remove(instance_id.as_str());
        }

        Ok(instance_id)
    }

    /// 启动工作流并在后台跑到结束（立即返回 instance_id）。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param definition - 图定义
    /// @param initial_context - 初始 Context
    /// @returns 实例 id
    pub async fn start_detached(
        &self,
        definition: WorkflowDefinition,
        initial_context: WorkflowContext,
    ) -> Result<InstanceId, WorkflowError> {
        let graph = DagBuilder::build(&definition)?;
        let instance_id = InstanceId::generate();

        let mut node_states = HashMap::new();
        let mut initial_pairs = Vec::new();
        for node_id in graph.nodes.keys() {
            let state = match node_id == &graph.start_id {
                true => NodeState::Ready,
                false => NodeState::Pending,
            };
            node_states.insert(node_id.clone(), state);
            initial_pairs.push((node_id.clone(), state));
        }

        let specs: Vec<&crate::definition::NodeSpec> = definition.nodes.iter().collect();
        self.checkpoint.create_instance(
            &instance_id,
            &definition,
            WorkflowState::Running,
            &initial_context,
            &specs,
            &initial_pairs,
        )?;

        let handle = SchedulerHandle {
            instance_id: instance_id.clone(),
            definition,
            graph,
            workflow_state: WorkflowState::Running,
            node_states,
            attempts: HashMap::new(),
            retry_states: HashMap::new(),
            context: initial_context,
            cancel_requested: false,
            pause_requested: false,
        };

        self.event_bus.publish(WorkflowEvent::WorkflowStarted {
            instance_id: instance_id.clone(),
            state: WorkflowState::Running,
        })?;

        let scheduler = Arc::clone(&self.scheduler);
        let handle_arc = Arc::new(Mutex::new(handle));
        {
            let mut guard = self.active.lock().await;
            guard.insert(instance_id.as_str().to_string(), Arc::clone(&handle_arc));
        }

        let active_map = Arc::clone(&self.active);
        let key = instance_id.as_str().to_string();
        tokio::spawn(async move {
            {
                let mut locked = handle_arc.lock().await;
                let _ = scheduler.run_until_idle(&mut locked).await;
            }
            let mut guard = active_map.lock().await;
            guard.remove(&key);
        });

        Ok(instance_id)
    }

    /// 请求暂停。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub async fn pause(&self, instance_id: &InstanceId) -> Result<(), WorkflowError> {
        let handle = self.active_handle(instance_id).await?;
        let mut guard = handle.lock().await;
        guard.pause_requested = true;
        Ok(())
    }

    /// 取消。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub async fn cancel(&self, instance_id: &InstanceId) -> Result<(), WorkflowError> {
        match self.active_handle(instance_id).await {
            Ok(handle) => {
                let mut guard = handle.lock().await;
                guard.cancel_requested = true;
                Ok(())
            }
            Err(_) => Err(WorkflowError::NotAllowed {
                message: "cancel only supported for in-process active runs in this version"
                    .to_string(),
            }),
        }
    }

    /// 恢复：后台继续调度，立即返回 Running。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub async fn resume(&self, instance_id: &InstanceId) -> Result<WorkflowState, WorkflowError> {
        let mut handle = self.recovery.load_handle(instance_id)?;
        match handle.workflow_state {
            WorkflowState::Paused
            | WorkflowState::Running
            | WorkflowState::Pausing
            | WorkflowState::Failing
            | WorkflowState::Cancelling => {
                handle.workflow_state = WorkflowState::Running;
            }
            other => {
                return Err(WorkflowError::NotAllowed {
                    message: format!("cannot resume from state {other}"),
                });
            }
        }
        handle.cancel_requested = false;
        handle.pause_requested = false;

        let scheduler = Arc::clone(&self.scheduler);
        let handle_arc = Arc::new(Mutex::new(handle));
        {
            let mut guard = self.active.lock().await;
            guard.insert(instance_id.as_str().to_string(), Arc::clone(&handle_arc));
        }

        let active_map = Arc::clone(&self.active);
        let key = instance_id.as_str().to_string();
        tokio::spawn(async move {
            {
                let mut locked = handle_arc.lock().await;
                let _ = scheduler.run_until_idle(&mut locked).await;
            }
            let mut guard = active_map.lock().await;
            guard.remove(&key);
        });

        Ok(WorkflowState::Running)
    }

    /// 可恢复列表。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn list_recoverable(
        &self,
    ) -> Result<Vec<ports::workflow_runtime::WfRtInstanceRecord>, WorkflowError> {
        self.recovery.list_recoverable()
    }

    /// 查询实例状态。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn get_instance_state(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Option<WorkflowState>, WorkflowError> {
        match self.checkpoint.get_instance(instance_id)? {
            Some(record) => Ok(WorkflowState::parse(&record.state)),
            None => Ok(None),
        }
    }

    /// Registry 引用（供外部注册更多 Executor）。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn registry(&self) -> &ExecutorRegistry {
        &self.registry
    }

    async fn active_handle(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Arc<Mutex<SchedulerHandle>>, WorkflowError> {
        let guard = self.active.lock().await;
        match guard.get(instance_id.as_str()) {
            Some(handle) => Ok(Arc::clone(handle)),
            None => Err(WorkflowError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            }),
        }
    }
}
