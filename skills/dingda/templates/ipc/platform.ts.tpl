// packages/platform/src/ipc/{{FEATURE}}.ts — skeleton
import { invoke } from "@tauri-apps/api/core";

export async function {{FEATURE}}{{COMMAND}}(
  _input: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  return invoke("{{FEATURE}}_{{COMMAND_SNAKE}}", _input);
}
