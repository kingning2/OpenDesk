/**
 * 将采集画布节点/边编译为 Workflow Runtime Definition JSON。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

import type { Edge, Node } from "@desk/ui";

/**
 * 采集步骤 kind（与 crawler-page 对齐）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export type CrawlerFlowKind = "source" | "generate" | "search" | "summary";

/**
 * 画布节点 data 最小形状。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export interface CrawlerFlowNodeData {
  kind: CrawlerFlowKind;
  [key: string]: unknown;
}

/**
 * kind → Runtime NodeType（snake_case）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param kind - 采集步骤
 * @returns NodeType 字符串
 */
function nodeTypeForKind(kind: CrawlerFlowKind): string {
  switch (kind) {
    case "source":
      return "delay";
    case "generate":
      return "crawler_generate";
    case "search":
      return "crawler_search";
    case "summary":
      return "crawler_summary";
    default:
      return "delay";
  }
}

/**
 * 从采集画布构建 Runtime Definition（自动补 Start / End）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param nodes - 画布节点
 * @param edges - 画布边
 * @param options - autoLoop 等
 * @returns 可 JSON.stringify 的 definition 对象
 */
export function buildCrawlerRuntimeDefinition(
  nodes: Node<CrawlerFlowNodeData>[],
  edges: Edge[],
  options?: { autoLoop?: boolean; restartNodeId?: string },
): {
  id: string;
  nodes: Array<{
    id: string;
    node_type: string;
    config: Record<string, unknown>;
    retry: { max_retry: number; strategy: string; base_delay_ms: number };
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
    branch?: string | null;
  }>;
  run_policy: "once" | { on_success_restart_from: { node_id: string } };
} {
  const startId = "rt-start";
  const endId = "rt-end";
  const canvasIds = new Set(nodes.map((node) => node.id));

  const runtimeNodes = [
    {
      id: startId,
      node_type: "start",
      config: {},
      retry: { max_retry: 0, strategy: "immediate", base_delay_ms: 0 },
    },
    ...nodes.map((node) => ({
      id: node.id,
      node_type: nodeTypeForKind(node.data.kind),
      config:
        node.data.kind === "source"
          ? { delay_ms: 0 }
          : ({} as Record<string, unknown>),
      retry: { max_retry: 0, strategy: "immediate", base_delay_ms: 0 },
    })),
    {
      id: endId,
      node_type: "end",
      config: {},
      retry: { max_retry: 0, strategy: "immediate", base_delay_ms: 0 },
    },
  ];

  const incoming = new Set<string>();
  const outgoing = new Set<string>();
  const runtimeEdges: Array<{
    id: string;
    source: string;
    target: string;
    branch?: string | null;
  }> = [];

  for (const edge of edges) {
    if (!canvasIds.has(edge.source) || !canvasIds.has(edge.target)) {
      continue;
    }
    incoming.add(edge.target);
    outgoing.add(edge.source);
    runtimeEdges.push({
      id: edge.id,
      source: edge.source,
      target: edge.target,
      branch: null,
    });
  }

  const roots = [...nodes.filter((node) => !incoming.has(node.id))];
  const leaves = [...nodes.filter((node) => !outgoing.has(node.id))];

  if (roots.length === 0 && nodes.length > 0) {
    roots.push(nodes[0]);
  }
  if (leaves.length === 0 && nodes.length > 0) {
    leaves.push(nodes[nodes.length - 1]);
  }

  for (const root of roots) {
    runtimeEdges.push({
      id: `e-${startId}-${root.id}`,
      source: startId,
      target: root.id,
      branch: null,
    });
  }
  for (const leaf of leaves) {
    runtimeEdges.push({
      id: `e-${leaf.id}-${endId}`,
      source: leaf.id,
      target: endId,
      branch: null,
    });
  }

  if (nodes.length === 0) {
    runtimeEdges.push({
      id: `e-${startId}-${endId}`,
      source: startId,
      target: endId,
      branch: null,
    });
  }

  const restartId =
    options?.restartNodeId ??
    nodes.find((node) => node.data.kind === "generate")?.id ??
    roots[0]?.id;

  const run_policy: "once" | { on_success_restart_from: { node_id: string } } =
    options?.autoLoop && restartId
      ? { on_success_restart_from: { node_id: restartId } }
      : "once";

  return {
    id: "crawler-canvas",
    nodes: runtimeNodes,
    edges: runtimeEdges,
    run_policy,
  };
}
