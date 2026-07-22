/**
 * TanStack Table column definitions for crawler results.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { useState } from "react";
import type { CrawlerChannelListRow } from "@desk/platform/ipc/crawler";
import { openExternalUrl } from "@desk/platform/window";
import { Button, Input, type ColumnDef } from "@desk/ui";

export interface CrawlerResultsColumnOptions {
  t: (key: string) => string;
  savingIds: ReadonlySet<number>;
  onSaveVerifiedEmail: (row: CrawlerChannelListRow, verifiedEmail: string) => Promise<void>;
}

function formatSubscribers(value?: number): string {
  if (value == null) {
    return "—";
  }
  return value.toLocaleString();
}

function channelHandle(row: CrawlerChannelListRow): string {
  if (row.custom_url?.trim()) {
    return row.custom_url.startsWith("@") ? row.custom_url : `@${row.custom_url}`;
  }
  return row.channel_id;
}

function channelUrl(row: CrawlerChannelListRow): string {
  if (row.custom_url?.trim()) {
    const handle = row.custom_url.replace(/^@/, "");
    return `https://www.youtube.com/@${handle}`;
  }
  return `https://www.youtube.com/channel/${row.channel_id}`;
}

/**
 * Inline verified-email editor for one table row.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function VerifiedEmailCell({
  row,
  saving,
  placeholder,
  onSave,
}: {
  row: CrawlerChannelListRow;
  saving: boolean;
  placeholder: string;
  onSave: (verifiedEmail: string) => Promise<void>;
}) {
  const [value, setValue] = useState(row.verified_email ?? "");

  return (
    <Input
      value={value}
      disabled={saving}
      placeholder={placeholder}
      className="h-8 w-[200px]"
      onChange={(event) => setValue(event.target.value)}
      onBlur={() => {
        if (value.trim() === (row.verified_email ?? "").trim()) {
          return;
        }
        void onSave(value);
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter") {
          event.currentTarget.blur();
        }
      }}
    />
  );
}

/**
 * Build crawler results table columns.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param options - i18n, save handler and saving state
 * @returns TanStack column definitions
 */
export function createCrawlerResultsColumns({
  t,
  savingIds,
  onSaveVerifiedEmail,
}: CrawlerResultsColumnOptions): ColumnDef<CrawlerChannelListRow, unknown>[] {
  return [
    {
      accessorKey: "title",
      header: () => t("crawler-results.columns.title"),
      meta: {
        headerClassName: "min-w-[200px]",
        cellClassName: "min-w-[200px] max-w-[280px]",
      },
      cell: ({ row }) => (
        <div className="truncate font-medium">{row.original.title}</div>
      ),
    },
    {
      id: "channel",
      header: () => t("crawler-results.columns.channel"),
      meta: {
        headerClassName: "min-w-[96px]",
        cellClassName: "min-w-[96px]",
      },
      cell: ({ row }) => (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="whitespace-nowrap"
          onClick={() => {
            void openExternalUrl(channelUrl(row.original));
          }}
        >
          {t("crawler-results.openChannel")}
        </Button>
      ),
    },
    {
      id: "handle",
      header: () => t("crawler-results.columns.handle"),
      meta: {
        headerClassName: "min-w-[120px]",
        cellClassName: "min-w-[120px] max-w-[180px]",
      },
      cell: ({ row }) => (
        <span className="block truncate text-muted-foreground">{channelHandle(row.original)}</span>
      ),
    },
    {
      accessorKey: "subscriber_count",
      header: () => t("crawler-results.columns.subscribers"),
      meta: {
        headerClassName: "min-w-[88px]",
        cellClassName: "min-w-[88px]",
      },
      cell: ({ row }) => (
        <span className="tabular-nums">{formatSubscribers(row.original.subscriber_count)}</span>
      ),
    },
    {
      accessorKey: "country",
      header: () => t("crawler-results.columns.country"),
      meta: {
        headerClassName: "min-w-[56px]",
        cellClassName: "min-w-[56px]",
      },
      cell: ({ row }) => row.original.country || "—",
    },
    {
      accessorKey: "email",
      header: () => t("crawler-results.columns.email"),
      meta: {
        headerClassName: "min-w-[160px]",
        cellClassName: "min-w-[160px] max-w-[220px]",
      },
      cell: ({ row }) => (
        <div className="truncate text-muted-foreground">{row.original.email || "—"}</div>
      ),
    },
    {
      id: "verified_email",
      header: () => t("crawler-results.columns.verifiedEmail"),
      meta: {
        headerClassName: "min-w-[220px]",
        cellClassName: "min-w-[220px]",
      },
      cell: ({ row }) => (
        <VerifiedEmailCell
          key={`${row.original.id}-${row.original.verified_email ?? ""}`}
          row={row.original}
          saving={savingIds.has(row.original.id)}
          placeholder={t("crawler-results.verifiedEmailPlaceholder")}
          onSave={(verifiedEmail) => onSaveVerifiedEmail(row.original, verifiedEmail)}
        />
      ),
    },
    {
      accessorKey: "email_status",
      header: () => t("crawler-results.columns.emailStatus"),
      meta: {
        headerClassName: "min-w-[112px]",
        cellClassName: "min-w-[112px]",
      },
      cell: ({ row }) =>
        row.original.email_status
          ? t(`crawler-results.emailStatus.${row.original.email_status}`)
          : "—",
    },
    {
      accessorKey: "keyword",
      header: () => t("crawler-results.columns.keyword"),
      meta: {
        headerClassName: "min-w-[120px]",
        cellClassName: "min-w-[120px] max-w-[180px]",
      },
      cell: ({ row }) => (
        <div className="truncate text-muted-foreground">{row.original.keyword}</div>
      ),
    },
  ];
}
