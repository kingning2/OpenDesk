export interface MailIpcTemplateSaveRequest {
  id?: string;
  name: string;
  template_intent: string;
  subject_template: string;
  body_text_template: string;
  body_html_template?: string;
  locale?: string;
  is_active?: boolean;
  sort_order?: number;
}
