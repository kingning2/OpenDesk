export interface WorkflowRuntimeIpcResumeResponse {
  instance_id: string;
  state: string;
  error?: string;
}
