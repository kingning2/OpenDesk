/**
 * 插件（解析工具）设置面板 — 嵌入 SettingsDialog。
 *
 * 以卡片形式展示 Pandoc / Tesseract / PDFium 三个解析工具：每张卡片带图标、
 * 用途说明、安装状态与下载按钮。状态经 `knowledge_tool_status` 拉取，
 * 进度经 `knowledge:tool/progress` 事件订阅。
 *
 * @author coisini
 * @created 2026-08-06
 */

import { useCallback, useEffect, useState } from "react";
import { Button, Card, LoadingState } from "@desk/ui";
import {
  FileText,
  FileType2,
  Loader2,
  ScanText,
  type LucideIcon,
} from "@desk/ui/icons";
import {
  knowledgeToolDownload,
  knowledgeToolStatus,
  listenKnowledgeToolProgress,
  type KnowledgeDownloadProgress,
  type KnowledgeToolStatus,
} from "@desk/platform/ipc/knowledge";
import { useT } from "../../i18n";

const toDisplayError = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

/** 各工具的卡片元数据：图标 + 用途说明文案 key。 */
const TOOL_META: Record<string, { icon: LucideIcon; descKey: string }> = {
  pandoc: { icon: FileText, descKey: "settings.toolDescPandoc" },
  tesseract: { icon: ScanText, descKey: "settings.toolDescTesseract" },
  pdfium: { icon: FileType2, descKey: "settings.toolDescPdfium" },
};

/** 下载进度百分比的展示文案。 */
function progressLabel(p: { bytes_downloaded: number; bytes_total: number }): string {
  if (!p.bytes_total) {
    return p.bytes_downloaded > 0 ? `${Math.round(p.bytes_downloaded / 1024)} KB` : "";
  }
  const percent = Math.min(100, Math.round((p.bytes_downloaded / p.bytes_total) * 100));
  return `${percent}%`;
}

/**
 * 单个解析工具卡片：图标 / 名称 / 状态 / 描述 + 下载按钮与进度条。
 *
 * @param tool - 工具状态
 * @param downloading - 该工具当前的下载进度（null 表示未在下载）
 * @param onDownload - 触发下载
 */
function ToolCard({
  tool,
  downloading,
  onDownload,
}: {
  tool: KnowledgeToolStatus;
  downloading: KnowledgeDownloadProgress | null;
  onDownload: (id: string) => void;
}) {
  const t = useT();
  const meta = TOOL_META[tool.id];
  const Icon = meta?.icon;
  const inProgress = downloading?.status === "downloading" || downloading?.status === "extracting";
  const percent = downloading ? progressLabel(downloading) : "";

  return (
    <Card variant="default" padding="md" className="flex flex-col gap-3">
      <div className="flex items-start gap-3">
        {Icon ? <Icon className="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden /> : null}
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-[length:var(--text-sm)] font-medium text-foreground">
              {tool.name}
            </span>
            {tool.installed ? (
              <span className="rounded-full bg-emerald-500/10 px-2 py-0.5 text-[length:var(--text-xs)] font-medium text-emerald-600 dark:text-emerald-400">
                {tool.version || t("settings.toolInstalled")}
              </span>
            ) : (
              <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] font-medium text-muted-foreground">
                {t("settings.toolNotInstalled")}
              </span>
            )}
          </div>
          {meta?.descKey ? (
            <p className="mt-1.5 text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
              {t(meta.descKey)}
            </p>
          ) : null}
        </div>
      </div>

      <div className="flex items-center gap-3">
        {inProgress ? (
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <div className="h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-[width] duration-150"
                style={{ width: percent }}
              />
            </div>
            <span className="shrink-0 text-[length:var(--text-xs)] text-muted-foreground">
              {percent || t("settings.toolDownloading")}
            </span>
          </div>
        ) : null}
        {tool.installed ? null : (
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="ml-auto"
            disabled={inProgress}
            onClick={() => onDownload(tool.id)}
          >
            {inProgress ? (
              <>
                <Loader2 className="size-3.5 animate-spin" aria-hidden />
                {t("settings.toolDownloading")}
              </>
            ) : (
              t("settings.toolDownload")
            )}
          </Button>
        )}
      </div>
    </Card>
  );
}

/**
 * 插件（解析工具）设置面板。
 *
 * @returns 面板节点
 */
export function PluginsSettingsPanel() {
  const t = useT();
  const [tools, setTools] = useState<KnowledgeToolStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [downloading, setDownloading] = useState<
    Record<string, KnowledgeDownloadProgress | null>
  >({});

  // 拉取工具状态；切换到此 tab 时重新挂载并刷新。用 rAF 延迟到下一帧，避免同步 setState。
  const refreshTools = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      const statuses = await knowledgeToolStatus();
      setTools(statuses);
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let frame = 0;
    frame = window.requestAnimationFrame(() => {
      void refreshTools();
    });
    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [refreshTools]);

  // 订阅下载进度；done / failed 后刷新工具状态。
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void listenKnowledgeToolProgress((payload) => {
      if (cancelled) {
        return;
      }
      setDownloading((prev) => ({ ...prev, [payload.tool]: payload }));
      if (payload.status === "done" || payload.status === "failed") {
        setDownloading((prev) => ({ ...prev, [payload.tool]: null }));
        void refreshTools();
      }
    }).then((dispose) => {
      if (cancelled) {
        dispose();
        return;
      }
      unlisten = dispose;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refreshTools]);

  const downloadTool = async (toolId: string) => {
    setError("");
    setDownloading((prev) => ({
      ...prev,
      [toolId]: { tool: toolId, bytes_downloaded: 0, bytes_total: 0, status: "downloading" },
    }));
    try {
      await knowledgeToolDownload({ tool: toolId });
    } catch (err) {
      setError(toDisplayError(err));
      setDownloading((prev) => ({ ...prev, [toolId]: null }));
    }
  };

  return (
    <section className="flex max-w-xl flex-col gap-6">
      <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {t("settings.pluginsDescription")}
      </p>

      {error ? (
        <p className="text-[length:var(--text-sm)] text-red-500">{error}</p>
      ) : null}

      {loading ? (
        <LoadingState label={t("settings.pluginsLoading")} />
      ) : (
        <div className="space-y-3">
          {tools.map((tool) => (
            <ToolCard
              key={tool.id}
              tool={tool}
              downloading={downloading[tool.id] ?? null}
              onDownload={(id) => void downloadTool(id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}
