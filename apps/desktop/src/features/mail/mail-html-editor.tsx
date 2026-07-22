/**
 * Plain-text + HTML compose editor with live HTML preview.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { memo, useState } from "react";
import { Button, cn } from "@desk/ui";

import { MailHtmlPreview } from "./mail-html-preview";

export type MailBodyEditorMode = "text" | "html";

/**
 * Dual-mode body editor: plain text tab + HTML source tab with live preview.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export const MailHtmlEditor = memo(function MailHtmlEditor({
  bodyText,
  bodyHtml,
  onBodyTextChange,
  onBodyHtmlChange,
  textLabel,
  htmlLabel,
  previewLabel,
  textPlaceholder,
  htmlPlaceholder,
  emptyPreviewLabel,
  className,
}: {
  bodyText: string;
  bodyHtml: string;
  onBodyTextChange: (value: string) => void;
  onBodyHtmlChange: (value: string) => void;
  textLabel: string;
  htmlLabel: string;
  previewLabel: string;
  textPlaceholder: string;
  htmlPlaceholder: string;
  emptyPreviewLabel: string;
  className?: string;
}) {
  const [mode, setMode] = useState<MailBodyEditorMode>(bodyHtml.trim() ? "html" : "text");

  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex gap-1">
        <Button
          type="button"
          size="sm"
          variant={mode === "text" ? "default" : "outline"}
          onClick={() => setMode("text")}
        >
          {textLabel}
        </Button>
        <Button
          type="button"
          size="sm"
          variant={mode === "html" ? "default" : "outline"}
          onClick={() => setMode("html")}
        >
          {htmlLabel}
        </Button>
      </div>

      {mode === "text" ? (
        <textarea
          value={bodyText}
          onChange={(event) => onBodyTextChange(event.target.value)}
          placeholder={textPlaceholder}
          className="min-h-32 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 font-sans text-[length:var(--text-sm)]"
        />
      ) : (
        <div className="grid gap-3 lg:grid-cols-2">
          <textarea
            value={bodyHtml}
            onChange={(event) => onBodyHtmlChange(event.target.value)}
            placeholder={htmlPlaceholder}
            spellCheck={false}
            className="min-h-48 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 font-mono text-[length:var(--text-xs)] leading-relaxed"
          />
          <div className="space-y-1.5">
            <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
              {previewLabel}
            </p>
            <MailHtmlPreview
              bodyHtml={bodyHtml}
              bodyText={bodyText}
              emptyLabel={emptyPreviewLabel}
              minHeight={192}
            />
          </div>
        </div>
      )}
    </div>
  );
});
