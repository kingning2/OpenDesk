export interface RuntimeLogEntry {
  schema_version: string;
  timestamp: string;
  level: string;
  source: string;
  logger: string;
  message: string;
  event?: string;
  feature?: string;
  trace_id?: string;
  task_id?: string;
  tenant_id?: string;
  attributes?: unknown;
  exception?: unknown;
}
