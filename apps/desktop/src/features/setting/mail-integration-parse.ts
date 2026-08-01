/**
 * Run user-authored parse scripts in the settings UI test runner.
 *
 * @author coisini
 * @created 2026-08-01
 */

export type ParsedOpenStatus = {
  openCount: number;
  openedAt: string | null;
};

/**
 * Execute `parseResponse(data)` from the user's script against API JSON.
 *
 * @author coisini
 * @created 2026-08-01
 */
export function runEmailReadParseScript(
  script: string,
  data: unknown,
): ParsedOpenStatus {
  const trimmed = script.trim();
  if (!trimmed) {
    throw new Error("parse_script_empty");
  }
  let runner: (data: unknown) => ParsedOpenStatus;
  try {
    runner = new Function(
      "data",
      `${trimmed}\nreturn parseResponse(data);`,
    ) as (data: unknown) => ParsedOpenStatus;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`parse_script_syntax: ${message}`);
  }
  const result = runner(data);
  if (!result || typeof result !== "object") {
    throw new Error("parse_script_invalid_return");
  }
  const openCount = Number((result as ParsedOpenStatus).openCount ?? 0);
  const openedAtRaw = (result as ParsedOpenStatus).openedAt;
  const openedAt =
    typeof openedAtRaw === "string" && openedAtRaw.trim() ? openedAtRaw.trim() : null;
  return {
    openCount: Number.isFinite(openCount) ? openCount : 0,
    openedAt,
  };
}
