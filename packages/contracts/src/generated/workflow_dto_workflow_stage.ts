export interface WorkflowDtoWorkflowStage {
  template_id: string;
  id: string;
  name: string;
  note?: string;
  ord: number;
  ai_level?: string;
  x?: number;
  y?: number;
  scripts_json: string;
  script_conds_json: string;
}
