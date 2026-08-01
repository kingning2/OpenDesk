/**
 * 工作流节点面板（可新增节点）条目定义。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

import type { WorkflowStepNodeData } from "./workflow-step-node";

/**
 * 左侧节点面板中的可添加模板。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export interface WorkflowPaletteItem {
  /** 模板稳定 id（拖拽 / 点击时回传）。 */
  id: string;
  /** 面板展示标题。 */
  label: string;
  /** 可选说明。 */
  description?: string;
  /** 落到画布上的 React Flow `type`，默认 `workflowStep`。 */
  nodeType?: string;
  /** 新节点的初始 data。 */
  defaultData: WorkflowStepNodeData;
}
