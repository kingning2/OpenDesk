/**
 * 工作流工作台公共组件导出。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

export { WorkflowWorkbench } from "./workflow-workbench";
export type {
  WorkflowWorkbenchProps,
  WorkflowNode,
  WorkflowEdge,
  Connection,
  XYPosition,
} from "./workflow-workbench";
export { WorkflowStepNode } from "./workflow-step-node";
export type { WorkflowStepNodeData, WorkflowStepTone } from "./workflow-step-node";
export type { WorkflowPaletteItem } from "./workflow-palette-item";

/** 透传 xyflow 常用 API，业务侧无需直接依赖 `@xyflow/react`。 */
export {
  MarkerType,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  useEdgesState,
  useNodesState,
} from "@xyflow/react";
export type { Edge, Node, NodeProps, NodeTypes, OnEdgesChange, OnNodesChange } from "@xyflow/react";
