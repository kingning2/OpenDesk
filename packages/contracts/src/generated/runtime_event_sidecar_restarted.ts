export interface RuntimeEventSidecarRestarted {
  event_id: string;
  occurred_at: string;
  port: number;
  attempt: number;
  reason?: string;
}
