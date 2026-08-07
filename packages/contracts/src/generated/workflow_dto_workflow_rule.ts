export interface WorkflowDtoWorkflowRule {
  id: string;
  name: string;
  from_stages_json: string;
  to_stage: string;
  trigger_keywords_json: string;
  trigger_tags_json: string;
  auto_reply: boolean;
  auto_advance: boolean;
  reply_script_id?: string;
}
