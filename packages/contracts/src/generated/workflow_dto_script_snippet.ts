export interface WorkflowDtoScriptSnippet {
  id: string;
  source_id: string;
  title: string;
  stage?: string;
  trigger_text?: string;
  description?: string;
  from_stage?: string;
  to_stage?: string;
  tags_json: string;
  body_text: string;
  category_l1?: string;
  category_l2?: string;
  needs_boss_input: boolean;
  boss_input_hint?: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
