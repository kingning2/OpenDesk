import { listenEvent } from "./index";

export const MONITOR_MATCH_EVENT = "app/monitor";

export interface MonitorMatchPayload {
  category: "monitor_match";
  taskId: string;
  taskName: string;
  itemId: string;
  title: string;
  url: string;
  priceText: string;
  reason: string;
}

export function listenMonitorMatch(
  handler: (payload: MonitorMatchPayload) => void,
) {
  return listenEvent<MonitorMatchPayload>(MONITOR_MATCH_EVENT, handler);
}
