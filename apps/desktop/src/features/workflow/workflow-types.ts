/**
 * Workflow canvas types + parser for the email-agent port.
 *
 * @author coisini
 * @created 2026-08-07
 */

export interface CanvasStage {
  id: string;
  name: string;
  note?: string;
  order?: number;
  scripts?: string[];
  aiLevel?: string;
  x?: number;
  y?: number;
  scriptConds?: string[];
}

export interface WorkflowCanvas {
  version?: string | number;
  updated?: string;
  stages: CanvasStage[];
  emailTypes?: unknown;
  archiveStates?: unknown;
  connections?: unknown;
  startNode?: unknown;
  endNode?: unknown;
}

/** Parse a `canvas_json` blob into a canvas, stages sorted by `order`. */
export function parseCanvas(canvasJson: string): WorkflowCanvas {
  try {
    const parsed = JSON.parse(canvasJson) as Partial<WorkflowCanvas>;
    const rawStages = Array.isArray(parsed.stages) ? (parsed.stages as CanvasStage[]) : [];
    const stages = [...rawStages].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
    return { ...parsed, stages };
  } catch {
    return { stages: [] };
  }
}
