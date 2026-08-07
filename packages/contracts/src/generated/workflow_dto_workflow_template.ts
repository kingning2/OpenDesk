export interface WorkflowDtoWorkflowTemplate {
  id: string;
  name: string;
  template_type: string;
  canvas_json: string;
  canvas_version?: string;
  canvas_updated?: string;
  binding_count?: number;
  created_at: string;
  updated_at: string;
}
