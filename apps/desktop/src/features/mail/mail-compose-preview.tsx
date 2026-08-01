/**
 * Live compose preview rail (subject / recipients / HTML body).
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import { memo } from "react";
import { cn } from "@desk/ui";

import { MailHtmlPreview } from "./mail-html-preview";

/**
 * Sticky right-rail preview for compose and reply forms.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */
export const MailComposePreview = memo(function MailComposePreview({
  fromLabel,
  toAddress,
  subject,
  bodyText,
  bodyHtml,
  previewTitle,
  fromLabelText,
  toLabelText,
  subjectLabelText,
  emptyLabel,
  className,
}: {
  fromLabel?: string;
  toAddress?: string;
  subject?: string;
  bodyText: string;
  bodyHtml: string;
  previewTitle: string;
  fromLabelText: string;
  toLabelText: string;
  subjectLabelText: string;
  emptyLabel: string;
  className?: string;
}) {
  return (
    <div className={cn("flex min-h-0 flex-col gap-3", className)}>
      <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
        {previewTitle}
      </p>
      <div className="space-y-2 rounded-[var(--radius-md)] border border-border/60 bg-muted/20 px-3 py-2 text-xs">
        {fromLabel ? (
          <p className="truncate text-muted-foreground">
            <span className="font-medium text-foreground">{fromLabelText}: </span>
            {fromLabel}
          </p>
        ) : null}
        {toAddress ? (
          <p className="truncate text-muted-foreground">
            <span className="font-medium text-foreground">{toLabelText}: </span>
            {toAddress}
          </p>
        ) : null}
        <p className="truncate font-medium text-foreground">
          <span className="text-muted-foreground">{subjectLabelText}: </span>
          {subject?.trim() || "—"}
        </p>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <MailHtmlPreview
          bodyHtml={bodyHtml}
          bodyText={bodyText}
          emptyLabel={emptyLabel}
          minHeight={320}
        />
      </div>
    </div>
  );
});
