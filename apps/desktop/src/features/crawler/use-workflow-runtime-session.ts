/**
 * 采集页对接 Workflow Runtime：active / resume / start / phase。
 *
 * @author coisini
 * @created 2026-07-23
 */

import { useCallback, useEffect, useState } from "react";
import type { WorkflowRuntimeEventPhase } from "@desk/contracts";
import {
  workflowRuntimeActive,
  workflowRuntimeCancel,
  workflowRuntimeResume,
  workflowRuntimeStart,
  type WorkflowRuntimeActiveInstance,
} from "@desk/platform/ipc/workflow-runtime";
import { listenWorkflowRuntimePhase } from "@desk/platform/ipc/workflow-runtime-events";
import type { WorkflowStepTone } from "@desk/ui";

/**
 * 将 Runtime phase 映射为画布 tone。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param state - phase.state
 * @returns tone
 */
export function toneFromRuntimeState(state?: string | null): WorkflowStepTone {
  switch (state) {
    case "running":
      return "running";
    case "succeeded":
    case "completed":
      return "done";
    case "failed":
      return "error";
    case "retry_waiting":
      return "warn";
    default:
      return "idle";
  }
}

/**
 * Workflow Runtime 会话钩子。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @returns 会话状态与操作
 */
export function useWorkflowRuntimeSession() {
  const [instanceId, setInstanceId] = useState<string | null>(null);
  const [runtimeState, setRuntimeState] = useState<string>("idle");
  const [recoverable, setRecoverable] = useState<WorkflowRuntimeActiveInstance[]>([]);
  const [lastPhase, setLastPhase] = useState<WorkflowRuntimeEventPhase | null>(null);
  const [nodeToneById, setNodeToneById] = useState<Record<string, WorkflowStepTone>>({});
  const [runtimeError, setRuntimeError] = useState<string | null>(null);

  const refreshActive = useCallback(async () => {
    const rows = await workflowRuntimeActive({});
    setRecoverable(rows);
    return rows;
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const rows = await workflowRuntimeActive({});
      if (!cancelled) {
        setRecoverable(rows);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenWorkflowRuntimePhase((payload) => {
      if (disposed) {
        return;
      }
      if (instanceId && payload.instance_id !== instanceId) {
        return;
      }
      setLastPhase(payload);
      if (payload.state) {
        setRuntimeState(payload.state);
      }
      if (payload.node_id && payload.state) {
        const tone = toneFromRuntimeState(payload.state);
        setNodeToneById((prev) => ({ ...prev, [payload.node_id as string]: tone }));
      }
      if (payload.kind === "workflow_completed" || payload.kind === "workflow_cancelled") {
        setInstanceId(null);
        void refreshActive();
      }
      if (payload.kind === "workflow_failed") {
        setRuntimeError(payload.message ?? "workflow failed");
        setInstanceId(null);
        void refreshActive();
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [instanceId, refreshActive]);

  /**
   * 用 definition JSON 启动 Runtime。
   *
   * @author coisini
   * @created 2026-07-23
   *
   * @param definitionJson - 图定义
   * @param contextJson - 可选初始 Context
   * @returns instance_id；失败返回 null
   */
  const startRuntime = useCallback(async (definitionJson: string, contextJson?: string) => {
    setRuntimeError(null);
    const response = await workflowRuntimeStart({
      definition_json: definitionJson,
      context_json: contextJson,
    });
    if (response.error || !response.instance_id) {
      setRuntimeError(response.error ?? "start failed");
      return null;
    }
    setInstanceId(response.instance_id);
    setRuntimeState(response.state || "running");
    setNodeToneById({});
    return response.instance_id;
  }, []);

  /**
   * 取消当前实例。
   *
   * @author coisini
   * @created 2026-07-23
   */
  const cancelRuntime = useCallback(async () => {
    if (!instanceId) {
      return;
    }
    const response = await workflowRuntimeCancel({ instance_id: instanceId });
    if (response.error) {
      setRuntimeError(response.error);
    }
    setInstanceId(null);
    await refreshActive();
  }, [instanceId, refreshActive]);

  /**
   * 恢复可恢复实例。
   *
   * @author coisini
   * @created 2026-07-23
   *
   * @param id - instance_id
   */
  const resumeRuntime = useCallback(
    async (id: string) => {
      setRuntimeError(null);
      const response = await workflowRuntimeResume({ instance_id: id });
      if (response.error) {
        setRuntimeError(response.error);
        return;
      }
      setInstanceId(id);
      setRuntimeState(response.state || "running");
      await refreshActive();
    },
    [refreshActive],
  );

  /**
   * 忽略可恢复实例（仅从横幅移除本地列表；不删库）。
   *
   * @author coisini
   * @created 2026-07-23
   *
   * @param id - instance_id
   */
  const dismissRecoverable = useCallback((id: string) => {
    setRecoverable((prev) => prev.filter((row) => row.instance_id !== id));
  }, []);

  return {
    instanceId,
    runtimeState,
    recoverable,
    lastPhase,
    nodeToneById,
    runtimeError,
    refreshActive,
    startRuntime,
    cancelRuntime,
    resumeRuntime,
    dismissRecoverable,
  };
}
