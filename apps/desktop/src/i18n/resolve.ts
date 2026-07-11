import type { Messages } from "./types";

export function resolveMessage(messages: Messages, key: string): string | undefined {
  const parts = key.split(".");
  let current: string | Messages = messages;

  for (const part of parts) {
    if (typeof current !== "object" || current === null || !(part in current)) {
      return undefined;
    }
    current = current[part];
  }

  return typeof current === "string" ? current : undefined;
}
