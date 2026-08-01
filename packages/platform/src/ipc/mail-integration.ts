/**
 * Mail external integration settings IPC.
 *
 * @author coisini
 * @created 2026-08-01
 */

import { invokeIpc } from "./invoke";

export interface MailEmailReadIntegrationConfig {
  enabled: boolean;
  api_base: string;
  pixel_path_template: string;
  query_path_template: string;
  parse_script: string;
}

/** Load email-read integration settings from DB (seeded by migration). */
export async function mailEmailReadIntegrationGet(): Promise<MailEmailReadIntegrationConfig> {
  return invokeIpc<MailEmailReadIntegrationConfig>("mail_email_read_integration_get");
}

/** Save email-read integration settings. */
export async function mailEmailReadIntegrationSave(
  config: MailEmailReadIntegrationConfig,
): Promise<MailEmailReadIntegrationConfig> {
  return invokeIpc<MailEmailReadIntegrationConfig>("mail_email_read_integration_save", {
    request: config,
  });
}

/** Probe query URL; returns raw JSON string. */
export async function mailEmailReadIntegrationProbe(input: {
  config: MailEmailReadIntegrationConfig;
  recipient_email: string;
  tracking_id: string;
}): Promise<{ response_json: string }> {
  return invokeIpc<{ response_json: string }>("mail_email_read_integration_probe", {
    request: input,
  });
}
