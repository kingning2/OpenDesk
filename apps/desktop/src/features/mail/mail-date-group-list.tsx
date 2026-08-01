/**
 * Collapsible date sections for inbox / sent mail lists.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import type { ReactNode } from "react";
import { useState } from "react";
import { ChevronRight } from "@desk/ui/icons";
import { cn } from "@desk/ui";
import type { MailMessageDateGroup } from "./mail-time";

/**
 * Collapsible mail list grouped by calendar day.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param props.groups - Date buckets from `groupMessagesByDate`
 * @param props.renderItem - Row renderer for each message
 * @param props.activeItemId - When set, expands the group containing this message
 * @returns Grouped list with toggleable date headers
 */
export function MailDateGroupList({
  groups,
  renderItem,
  activeItemId,
}: {
  groups: MailMessageDateGroup[];
  renderItem: (message: MailMessageDateGroup["items"][number]) => ReactNode;
  activeItemId?: string;
}) {
  const [collapsedKeys, setCollapsedKeys] = useState<Set<string>>(() => new Set());
  const activeGroupKey = activeItemId
    ? groups.find((group) => group.items.some((item) => item.id === activeItemId))?.key
    : undefined;

  function toggleGroup(key: string) {
    setCollapsedKeys((current) => {
      const next = new Set(current);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  }

  return (
    <>
      {groups.map((group) => {
        const expanded = group.key === activeGroupKey || !collapsedKeys.has(group.key);
        return (
          <div key={group.key}>
            <button
              type="button"
              className="sticky top-0 z-10 flex w-full items-center gap-1.5 border-b border-border/40 bg-muted/40 px-3 py-1.5 text-left text-[10px] font-medium text-muted-foreground backdrop-blur-sm transition-colors hover:bg-muted/60"
              aria-expanded={expanded}
              onClick={() => toggleGroup(group.key)}
            >
              <ChevronRight
                className={cn(
                  "size-3 shrink-0 transition-transform duration-150 ease-out",
                  expanded && "rotate-90",
                )}
              />
              <span className="min-w-0 truncate">{group.label}</span>
              <span className="shrink-0 text-muted-foreground/70">({group.items.length})</span>
            </button>
            {expanded ? group.items.map((item) => renderItem(item)) : null}
          </div>
        );
      })}
    </>
  );
}
