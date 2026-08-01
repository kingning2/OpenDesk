export interface WorkflowRuntimeEventPhase {
  kind: string;
  instance_id: string;
  node_id?: string;
  state?: string;
  message?: string;
  context_version?: number;
}
