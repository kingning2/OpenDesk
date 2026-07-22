import { invoke } from "@tauri-apps/api/core";
import type {
  WorkflowDtoScriptSnippet,
  WorkflowIpcSnippetDeleteRequest,
  WorkflowIpcSnippetListRequest,
  WorkflowIpcSnippetListResponse,
  WorkflowIpcSnippetSaveRequest,
} from "@desk/contracts";

export type ScriptSnippet = WorkflowDtoScriptSnippet;

/**
 * List script snippets with optional filters.
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param input - optional category_l1, category_l2, query filters
 * @returns Parsed snippet array and total count.
 */
export async function workflowSnippetList(
  input?: WorkflowIpcSnippetListRequest,
): Promise<{ items: ScriptSnippet[]; total: number }> {
  const response = await invoke<WorkflowIpcSnippetListResponse>("workflow_snippet_list", {
    request: input ?? {},
  });
  try {
    const parsed = JSON.parse(response.snippets_json ?? "[]") as ScriptSnippet[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Create or update a script snippet.
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param input - snippet fields; omit id to create
 * @returns Updated list filtered by the same category.
 */
export async function workflowSnippetSave(
  input: WorkflowIpcSnippetSaveRequest,
): Promise<{ items: ScriptSnippet[]; total: number }> {
  const response = await invoke<WorkflowIpcSnippetListResponse>("workflow_snippet_save", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.snippets_json ?? "[]") as ScriptSnippet[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Delete a script snippet by id.
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param input - { id } of snippet to delete
 * @returns Updated full list after deletion.
 */
export async function workflowSnippetDelete(
  input: WorkflowIpcSnippetDeleteRequest,
): Promise<{ items: ScriptSnippet[]; total: number }> {
  const response = await invoke<WorkflowIpcSnippetListResponse>("workflow_snippet_delete", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.snippets_json ?? "[]") as ScriptSnippet[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}
