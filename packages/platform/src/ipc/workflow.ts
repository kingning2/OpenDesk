import { invokeIpc } from "./invoke";
import type {
  WorkflowDtoWorkflowBinding,
  WorkflowDtoWorkflowRule,
  WorkflowDtoWorkflowScript,
  WorkflowDtoWorkflowTemplate,
  WorkflowIpcScriptListRequest,
} from "@desk/contracts";

export type WorkflowTemplate = WorkflowDtoWorkflowTemplate;
export type WorkflowBinding = WorkflowDtoWorkflowBinding;
export type WorkflowRule = WorkflowDtoWorkflowRule;
export type WorkflowScript = WorkflowDtoWorkflowScript;

/** List workflow templates with per-template binding counts. */
export async function workflowTemplateList(): Promise<{
  items: WorkflowTemplate[];
  total: number;
}> {
  const response = await invokeIpc<{
    templates_json: string;
    total: number;
  }>("workflow_template_list");
  try {
    const parsed = JSON.parse(response.templates_json ?? "[]") as WorkflowTemplate[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/** Fetch one workflow template by id (throws when not found). */
export async function workflowTemplateGet(id: string): Promise<WorkflowTemplate> {
  const response = await invokeIpc<{ template_json: string }>("workflow_template_get", {
    request: { id },
  });
  return JSON.parse(response.template_json) as WorkflowTemplate;
}

/** List all workflow account → template bindings. */
export async function workflowBindingList(): Promise<{
  items: WorkflowBinding[];
  total: number;
}> {
  const response = await invokeIpc<{
    bindings_json: string;
    total: number;
  }>("workflow_binding_list");
  try {
    const parsed = JSON.parse(response.bindings_json ?? "[]") as WorkflowBinding[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/** List all workflow routing rules. */
export async function workflowRuleList(): Promise<{ items: WorkflowRule[]; total: number }> {
  const response = await invokeIpc<{ rules_json: string; total: number }>("workflow_rule_list");
  try {
    const parsed = JSON.parse(response.rules_json ?? "[]") as WorkflowRule[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/** List workflow scripts with optional category / free-text filters. */
export async function workflowScriptList(
  input?: WorkflowIpcScriptListRequest,
): Promise<{ items: WorkflowScript[]; total: number }> {
  const response = await invokeIpc<{ scripts_json: string; total: number }>("workflow_script_list", {
    request: input ?? {},
  });
  try {
    const parsed = JSON.parse(response.scripts_json ?? "[]") as WorkflowScript[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}
