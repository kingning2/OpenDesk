/**
 * 工作流步骤节点（画布默认节点）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";

import { cn } from "../../lib/cn";

/**
 * 步骤节点状态色。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export type WorkflowStepTone = "idle" | "running" | "done" | "warn" | "error";

/**
 * 步骤节点展示数据。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export interface WorkflowStepNodeData extends Record<string, unknown> {
  /** 节点标题。 */
  title: string;
  /** 副标题。 */
  subtitle?: string;
  /** 状态摘要。 */
  value?: string;
  /** 运行态色调。 */
  tone?: WorkflowStepTone;
  /** 是否选中（由工作台注入）。 */
  selected?: boolean;
}

/**
 * 默认工作流步骤节点。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props - React Flow 节点 props
 * @returns 节点 UI
 */
export function WorkflowStepNode({
  data,
  selected,
}: NodeProps<Node<WorkflowStepNodeData>>) {
  let toneClass = "border-border bg-card/90";
  switch (data.tone) {
    case "running":
      toneClass = "border-sky-500/40 bg-sky-500/10";
      break;
    case "done":
      toneClass = "border-emerald-500/40 bg-emerald-500/10";
      break;
    case "warn":
      toneClass = "border-amber-500/40 bg-amber-500/10";
      break;
    case "error":
      toneClass = "border-red-500/40 bg-red-500/10";
      break;
    default:
      break;
  }

  const isSelected = selected || Boolean(data.selected);

  return (
    <div
      className={cn(
        "w-[220px] rounded-[var(--radius-lg)] border px-4 py-3 shadow-sm transition-[box-shadow,border-color,transform] duration-150 ease-out",
        toneClass,
        isSelected
          ? "ring-2 ring-primary/60 ring-offset-2 ring-offset-background"
          : "hover:shadow-md",
      )}
    >
      <Handle
        type="target"
        position={Position.Left}
        className="!size-2.5 !border-2 !border-background !bg-primary"
      />
      {data.subtitle ? (
        <div className="text-xs text-muted-foreground">{data.subtitle}</div>
      ) : null}
      <div className={cn("text-sm font-semibold", data.subtitle ? "mt-1" : undefined)}>
        {data.title}
      </div>
      {data.value ? (
        <div className="mt-2 text-[length:var(--text-sm)] leading-snug text-muted-foreground">
          {data.value}
        </div>
      ) : null}
      <Handle
        type="source"
        position={Position.Right}
        className="!size-2.5 !border-2 !border-background !bg-primary"
      />
    </div>
  );
}
