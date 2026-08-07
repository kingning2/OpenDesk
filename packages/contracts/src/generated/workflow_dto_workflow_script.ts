export interface WorkflowDtoWorkflowScript {
  id: string;
  stage?: string;
  category_l1?: string;
  category_l2?: string;
  trigger_text?: string;
  description?: string;
  from_stage?: string;
  to_stage?: string;
  tags_json: string;
  content: string;
  needs_boss_input: boolean;
  boss_input_hint?: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
