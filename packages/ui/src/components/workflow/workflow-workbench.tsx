/**
 * 工作流工作台（Coze 式布局）公共组件。
 *
 * 负责：
 * - 左侧节点面板（点击 / 拖入新增）
 * - 中央画布（基于 @xyflow/react：Controls / MiniMap / Background / 拖拽缩放 / 连线）
 * - 右侧选中节点配置区（`renderConfig`）
 * - 底部分步日志区（`renderLogs`）
 * - 底部执行操作区（`footer`，如 Start / Stop）
 *
 * 选型说明：业界标准节点编辑基座为 React Flow（`@xyflow/react`）；
 * 商业 Workflow Builder SDK / AntV X6 / Rete 迁移成本更高，本仓库已用 xyflow，
 * 因此沉淀为本公共组件供各 Feature 复用。
 *
 * 受控模式约束：`nodes` / `edges` 必须是状态本体（经 `onNodesChange` / `onEdgesChange`
 * 用 `applyNodeChanges` 更新），禁止在传入前每渲 `map` 出新对象数组，否则会与
 * React Flow 内部 `setNodes` 形成 Maximum update depth 死循环。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

import {
  Background,
  Controls,
  MiniMap,
  Panel,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type ColorMode,
  type Connection,
  type Edge,
  type Node,
  type OnConnect,
  type OnEdgesChange,
  type OnNodesChange,
  type NodeTypes,
  type OnSelectionChangeParams,
  type XYPosition,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import * as React from "react";

import { cn } from "../../lib/cn";
import { Button } from "../button";
import { ErrorBoundary } from "../error-boundary";
import { ScrollArea } from "../scroll-area";
import type { WorkflowPaletteItem } from "./workflow-palette-item";
import { WorkflowStepNode, type WorkflowStepNodeData } from "./workflow-step-node";

const DEFAULT_NODE_TYPES: NodeTypes = {
  workflowStep: WorkflowStepNode,
};

const PALETTE_DND_MIME = "application/opendesk-workflow-palette";

const FIT_VIEW_OPTIONS = { padding: 0.28, minZoom: 0.4, maxZoom: 1.4 };

const PRO_OPTIONS = { hideAttribution: true };

const DELETE_KEY_CODE = ["Backspace", "Delete"];

/**
 * 工作流工作台属性。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export interface WorkflowWorkbenchProps {
  /** 画布节点（默认 type=`workflowStep`）。必须是受控状态本体，勿每渲派生新数组。 */
  nodes: Node<WorkflowStepNodeData>[];
  /** 画布边。必须是受控状态本体，勿每渲派生新数组。 */
  edges: Edge[];
  /** 当前选中节点 id。 */
  selectedNodeId?: string | null;
  /**
   * 选中节点变化回调。
   *
   * @param nodeId - 新选中节点；取消选中时为 null
   */
  onSelectedNodeIdChange?: (nodeId: string | null) => void;
  /** 节点变更（拖拽位置 / 删除 / dimensions 等，需 applyNodeChanges）。 */
  onNodesChange?: OnNodesChange<Node<WorkflowStepNodeData>>;
  /** 边变更。 */
  onEdgesChange?: OnEdgesChange<Edge>;
  /**
   * 新增连线。
   *
   * @param connection - 源/目标连接
   */
  onConnect?: OnConnect;
  /**
   * 从节点面板新增节点。
   *
   * @param item - 面板模板
   * @param position - 落点（画布坐标）
   */
  onAddNode?: (item: WorkflowPaletteItem, position: XYPosition) => void;
  /** 左侧可添加节点模板；为空则不显示节点面板。 */
  paletteItems?: WorkflowPaletteItem[];
  /** 左侧节点面板标题。 */
  paletteTitle?: string;
  /** 左侧节点面板操作提示。 */
  paletteHint?: string;
  /** 自定义节点类型表；默认含 `workflowStep`。 */
  nodeTypes?: NodeTypes;
  /**
   * 右侧配置区渲染。
   *
   * @param ctx - 选中节点上下文
   * @returns 配置面板内容
   */
  renderConfig?: (ctx: {
    nodeId: string | null;
    node: Node<WorkflowStepNodeData> | undefined;
  }) => React.ReactNode;
  /**
   * 底部日志区渲染（应按当前步骤过滤）。
   *
   * @param ctx - 选中节点上下文
   * @returns 日志面板内容
   */
  renderLogs?: (ctx: {
    nodeId: string | null;
    node: Node<WorkflowStepNodeData> | undefined;
  }) => React.ReactNode;
  /** 底部执行区（开始 / 停止等）。 */
  footer?: React.ReactNode;
  /** 画布标题。 */
  canvasTitle?: string;
  /** 画布说明。 */
  canvasDescription?: string;
  /** 右侧配置标题。 */
  configTitle?: string;
  /** 底部日志标题。 */
  logsTitle?: string;
  /** 左侧节点面板宽度（px）。 */
  paletteWidth?: number;
  /** 右侧配置宽度（px）。 */
  configWidth?: number;
  /** 底部日志高度（px）。 */
  logsHeight?: number;
  /** 明暗模式。 */
  colorMode?: ColorMode;
  /** 根节点 className。 */
  className?: string;
  /** 未选中节点时的配置占位。 */
  emptyConfig?: React.ReactNode;
  /** 日志为空时的占位。 */
  emptyLogs?: React.ReactNode;
  /** 是否允许拖拽节点。 */
  nodesDraggable?: boolean;
  /** 是否允许连线。 */
  nodesConnectable?: boolean;
  /** 是否允许删除节点（Delete / Backspace）。 */
  nodesDeletable?: boolean;
  /** 是否允许删除边。 */
  edgesDeletable?: boolean;
}

/**
 * 内部画布：需要 `ReactFlowProvider` 才能用 `screenToFlowPosition`。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function WorkflowCanvas({
  nodes,
  edges,
  selectedNodeId,
  onSelectedNodeIdChange,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onAddNode,
  nodeTypes,
  canvasTitle,
  canvasDescription,
  colorMode,
  nodesDraggable,
  nodesConnectable,
  nodesDeletable,
  edgesDeletable,
}: {
  nodes: Node<WorkflowStepNodeData>[];
  edges: Edge[];
  selectedNodeId: string | null;
  onSelectedNodeIdChange?: (nodeId: string | null) => void;
  onNodesChange?: OnNodesChange<Node<WorkflowStepNodeData>>;
  onEdgesChange?: OnEdgesChange<Edge>;
  onConnect?: OnConnect;
  onAddNode?: (item: WorkflowPaletteItem, position: XYPosition) => void;
  nodeTypes?: NodeTypes;
  canvasTitle?: string;
  canvasDescription?: string;
  colorMode: ColorMode;
  nodesDraggable: boolean;
  nodesConnectable: boolean;
  nodesDeletable: boolean;
  edgesDeletable: boolean;
}) {
  const { screenToFlowPosition } = useReactFlow();

  const mergedNodeTypes = React.useMemo(
    () => ({ ...DEFAULT_NODE_TYPES, ...nodeTypes }),
    [nodeTypes],
  );

  const handleSelectionChange = React.useCallback(
    (params: OnSelectionChangeParams) => {
      const nextId = params.nodes[0]?.id ?? null;
      if (nextId === selectedNodeId) {
        return;
      }
      onSelectedNodeIdChange?.(nextId);
    },
    [onSelectedNodeIdChange, selectedNodeId],
  );

  const handleDragOver = React.useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleDrop = React.useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const raw = event.dataTransfer.getData(PALETTE_DND_MIME);
      if (!raw || !onAddNode) {
        return;
      }
      try {
        const item = JSON.parse(raw) as WorkflowPaletteItem;
        const position = screenToFlowPosition({
          x: event.clientX,
          y: event.clientY,
        });
        onAddNode(item, position);
      } catch {
        // ignore malformed payload
      }
    },
    [onAddNode, screenToFlowPosition],
  );

  return (
    <ReactFlow
      fitView
      fitViewOptions={FIT_VIEW_OPTIONS}
      colorMode={colorMode}
      nodes={nodes}
      edges={edges}
      nodeTypes={mergedNodeTypes}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      onSelectionChange={handleSelectionChange}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      nodesDraggable={nodesDraggable}
      nodesConnectable={nodesConnectable}
      elementsSelectable
      deleteKeyCode={nodesDeletable || edgesDeletable ? DELETE_KEY_CODE : null}
      selectNodesOnDrag={false}
      panOnDrag
      zoomOnScroll
      proOptions={PRO_OPTIONS}
      className="bg-transparent"
    >
      <Background gap={18} color="var(--color-border)" />
      <Controls showInteractive={false} />
      <MiniMap
        pannable
        zoomable
        className="!bg-card/80 !border-border"
        maskColor="color-mix(in oklab, var(--color-background) 55%, transparent)"
      />
      {canvasTitle || canvasDescription ? (
        <Panel
          position="top-left"
          className="rounded-[var(--radius-md)] border border-border bg-card/90 px-3 py-2 shadow-sm backdrop-blur-sm"
        >
          {canvasTitle ? (
            <div className="text-[length:var(--text-sm)] font-semibold">{canvasTitle}</div>
          ) : null}
          {canvasDescription ? (
            <p className="mt-0.5 text-[length:var(--text-xs)] text-muted-foreground">
              {canvasDescription}
            </p>
          ) : null}
        </Panel>
      ) : null}
    </ReactFlow>
  );
}

/**
 * Coze 式工作流工作台：节点面板 + 画布 + 右侧配置 + 底部分步日志 + 执行栏。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props - 见 {@link WorkflowWorkbenchProps}
 * @returns 工作台布局
 */
export function WorkflowWorkbench({
  nodes,
  edges,
  selectedNodeId = null,
  onSelectedNodeIdChange,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onAddNode,
  paletteItems = [],
  paletteTitle = "Nodes",
  paletteHint = "Click or drag onto canvas",
  nodeTypes,
  renderConfig,
  renderLogs,
  footer,
  canvasTitle,
  canvasDescription,
  configTitle,
  logsTitle,
  paletteWidth = 220,
  configWidth = 340,
  logsHeight = 200,
  colorMode = "light",
  className,
  emptyConfig,
  emptyLogs,
  nodesDraggable = true,
  nodesConnectable = true,
  nodesDeletable = true,
  edgesDeletable = true,
}: WorkflowWorkbenchProps) {
  const selectedNode = React.useMemo(
    () => nodes.find((node) => node.id === selectedNodeId),
    [nodes, selectedNodeId],
  );

  const configBody =
    renderConfig?.({ nodeId: selectedNodeId, node: selectedNode }) ??
    (selectedNodeId ? null : emptyConfig);
  const logsBody =
    renderLogs?.({ nodeId: selectedNodeId, node: selectedNode }) ?? emptyLogs;

  const showPalette = paletteItems.length > 0 && Boolean(onAddNode);

  function handlePaletteDragStart(event: React.DragEvent, item: WorkflowPaletteItem) {
    event.dataTransfer.setData(PALETTE_DND_MIME, JSON.stringify(item));
    event.dataTransfer.effectAllowed = "move";
  }

  function handlePaletteClick(item: WorkflowPaletteItem) {
    if (!onAddNode) {
      return;
    }
    const offset = (nodes.length % 6) * 36;
    onAddNode(item, {
      x: 80 + offset + Math.floor(nodes.length / 6) * 40,
      y: 100 + offset,
    });
  }

  return (
    <ErrorBoundary title="工作流画布错误" className={className}>
      <div className={cn("flex min-h-0 flex-1 overflow-hidden", className)}>
        {showPalette ? (
          <aside
            className="flex shrink-0 flex-col border-r border-border bg-muted/20"
            style={{ width: paletteWidth }}
          >
            <div className="shrink-0 border-b border-border/70 px-3 py-3">
              <div className="text-[length:var(--text-sm)] font-semibold">{paletteTitle}</div>
              <p className="mt-0.5 text-[length:var(--text-xs)] text-muted-foreground">
                {paletteHint}
              </p>
            </div>
            <ScrollArea className="min-h-0 flex-1">
              <div className="space-y-2 p-3">
                {paletteItems.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    draggable
                    onDragStart={(event) => handlePaletteDragStart(event, item)}
                    onClick={() => handlePaletteClick(item)}
                    className="w-full cursor-grab rounded-[var(--radius-md)] border border-border bg-card/80 px-3 py-2 text-left transition-[border-color,box-shadow,transform] duration-150 ease-out hover:border-primary/40 hover:shadow-sm active:cursor-grabbing active:scale-[0.98]"
                  >
                    <div className="text-[length:var(--text-sm)] font-medium">{item.label}</div>
                    {item.description ? (
                      <p className="mt-0.5 text-[length:var(--text-xs)] text-muted-foreground">
                        {item.description}
                      </p>
                    ) : null}
                  </button>
                ))}
              </div>
            </ScrollArea>
          </aside>
        ) : null}

        <div className="flex min-w-0 flex-1 flex-col">
          <div className="relative min-h-0 flex-1">
            <ReactFlowProvider>
              <WorkflowCanvas
                nodes={nodes}
                edges={edges}
                selectedNodeId={selectedNodeId}
                onSelectedNodeIdChange={onSelectedNodeIdChange}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                onAddNode={onAddNode}
                nodeTypes={nodeTypes}
                canvasTitle={canvasTitle}
                canvasDescription={canvasDescription}
                colorMode={colorMode}
                nodesDraggable={nodesDraggable}
                nodesConnectable={nodesConnectable}
                nodesDeletable={nodesDeletable}
                edgesDeletable={edgesDeletable}
              />
            </ReactFlowProvider>
          </div>

          {footer ? (
            <div className="flex shrink-0 items-center justify-end gap-2 border-t border-border bg-card/40 px-4 py-3">
              {footer}
            </div>
          ) : null}

          <div
            className="flex shrink-0 flex-col border-t border-border bg-background"
            style={{ height: logsHeight }}
          >
            <div className="flex shrink-0 items-center justify-between border-b border-border/70 px-4 py-2">
              <div className="text-[length:var(--text-sm)] font-medium">
                {logsTitle}
                {selectedNode?.data.title ? (
                  <span className="ml-2 text-xs font-normal text-muted-foreground">
                    · {selectedNode.data.title}
                  </span>
                ) : null}
              </div>
              {selectedNodeId && nodesDeletable ? (
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="text-muted-foreground"
                  onClick={() => {
                    onNodesChange?.([
                      {
                        type: "remove",
                        id: selectedNodeId,
                      },
                    ]);
                    onSelectedNodeIdChange?.(null);
                  }}
                >
                  Delete
                </Button>
              ) : null}
            </div>
            <ScrollArea className="min-h-0 flex-1">
              <div className="p-3">{logsBody}</div>
            </ScrollArea>
          </div>
        </div>

        <aside
          className="flex shrink-0 flex-col border-l border-border bg-card/20"
          style={{ width: configWidth }}
        >
          <div className="shrink-0 border-b border-border/70 px-4 py-3">
            <div className="text-[length:var(--text-sm)] font-semibold">
              {configTitle}
              {selectedNode?.data.title ? (
                <span className="ml-2 text-xs font-normal text-muted-foreground">
                  · {selectedNode.data.title}
                </span>
              ) : null}
            </div>
          </div>
          <ScrollArea className="min-h-0 flex-1">
            <div className="p-4">{configBody}</div>
          </ScrollArea>
        </aside>
      </div>
    </ErrorBoundary>
  );
}

export type { WorkflowStepNodeData };
export type { WorkflowPaletteItem } from "./workflow-palette-item";
export type { Node as WorkflowNode, Edge as WorkflowEdge } from "@xyflow/react";
export type { Connection, XYPosition };
