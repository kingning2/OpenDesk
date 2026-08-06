/**
 * Knowledge Feature — 文档导入、列表与删除。
 *
 * 导入由平台层弹原生文件选择窗口（`@desk/platform` 的 `pickDocumentFile`），选中的
 * 文件路径经 IPC 入队 `background_job`，由 `opendesk-worker` 进程解析为 Markdown 并
 * 向量化入库；完成后主进程推 `knowledge:import/updated` 事件刷新列表。
 *
 * @author coisini
 */

import {
  Button,
  Card,
  IconButton,
  LoadingState,
  PageScaffold,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
} from "@desk/ui";
import { FileText, Loader2, Plus, Trash2 } from "@desk/ui/icons";

import { useT } from "../../i18n";
import {
  useKnowledge,
  type KnowledgeDocument,
  type KnowledgeToolStatus,
} from "./use-knowledge";

/** 文档来源类型显示名。 */
function sourceTypeLabel(type: string): string {
  return type.toUpperCase();
}

/** 工具下载进度百分比的展示文案。 */
function progressLabel(p: { bytes_downloaded: number; bytes_total: number }): string {
  if (!p.bytes_total) {
    return p.bytes_downloaded > 0 ? `${Math.round(p.bytes_downloaded / 1024)} KB` : "";
  }
  const percent = Math.min(100, Math.round((p.bytes_downloaded / p.bytes_total) * 100));
  return `${percent}%`;
}

/**
 * 单个解析工具行：状态 / 版本 / 下载按钮 + 进度条。
 *
 * @param tool - 工具状态
 * @param downloading - 该工具当前的下载进度（null 表示未在下载）
 * @param onDownload - 触发下载
 */
function ToolRow({
  tool,
  downloading,
  onDownload,
}: {
  tool: KnowledgeToolStatus;
  downloading: { bytes_downloaded: number; bytes_total: number; status: string } | null;
  onDownload: (id: string) => void;
}) {
  const t = useT();
  const inProgress = downloading?.status === "downloading" || downloading?.status === "extracting";
  const percent = downloading ? progressLabel(downloading) : "";

  return (
    <div className="flex items-center gap-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-[length:var(--text-sm)] font-medium">{tool.name}</span>
          {tool.installed ? (
            <span className="text-[length:var(--text-xs)] text-emerald-600 dark:text-emerald-400">
              {tool.version || t("knowledge.toolInstalled")}
            </span>
          ) : (
            <span className="text-[length:var(--text-xs)] text-muted-foreground">
              {t("knowledge.toolNotInstalled")}
            </span>
          )}
        </div>
        {inProgress ? (
          <div className="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-muted">
            <div
              className="h-full rounded-full bg-primary transition-[width] duration-150"
              style={{ width: percent }}
            />
          </div>
        ) : null}
      </div>
      {tool.installed ? null : (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={inProgress}
          onClick={() => onDownload(tool.id)}
        >
          {inProgress ? (
            <>
              <Loader2 className="mr-1.5 size-3.5 animate-spin" aria-hidden />
              {percent || t("knowledge.toolDownloading")}
            </>
          ) : (
            t("knowledge.toolDownload")
          )}
        </Button>
      )}
    </div>
  );
}

/**
 * 单个文档行。
 *
 * @param doc - 文档记录
 * @param deleting - 是否正在删除该文档
 * @param onDelete - 删除回调
 */
function DocumentRow({
  doc,
  deleting,
  onDelete,
}: {
  doc: KnowledgeDocument;
  deleting: boolean;
  onDelete: (id: string) => void;
}) {
  const t = useT();
  const time = doc.created_at
    ? new Date(doc.created_at).toLocaleString()
    : "-";
  return (
    <Card variant="solid" padding="sm">
      <div className="flex items-center gap-3">
        <div className="flex min-w-0 flex-1 items-center gap-3">
          <FileText className="size-4 shrink-0 text-muted-foreground" aria-hidden />
          <div className="min-w-0">
            <p className="truncate text-[length:var(--text-sm)] font-medium">{doc.name}</p>
            <p className="truncate text-[length:var(--text-xs)] text-muted-foreground">
              {sourceTypeLabel(doc.source_type)} · {doc.chunk_count} {t("knowledge.chunks")} · {time}
            </p>
          </div>
        </div>
        <IconButton
          label={t("knowledge.delete")}
          disabled={deleting}
          onClick={() => onDelete(doc.id)}
        >
          {deleting ? <Loader2 className="size-4 animate-spin" /> : <Trash2 className="size-4" />}
        </IconButton>
      </div>
    </Card>
  );
}

/**
 * Knowledge 页面。
 *
 * @author coisini
 *
 * @returns 页面节点
 */
export function KnowledgePage() {
  const t = useT();
  const {
    documents,
    loading,
    importing,
    deletingId,
    error,
    pickFile,
    remove,
    clearError,
    tools,
    toolsLoading,
    downloading,
    downloadTool,
  } = useKnowledge();

  return (
    <PageScaffold fill containerPadding="none">
      <div className="flex h-full min-h-0 flex-col">
        <WorkspaceSplitToolbar className="px-4 py-2">
          <WorkspaceSplitTitle className="text-sm">{t("knowledge.title")}</WorkspaceSplitTitle>
          <div className="ml-auto flex items-center gap-2">
            <Button
              type="button"
              size="sm"
              disabled={importing}
              onClick={() => void pickFile()}
            >
              {importing ? (
                <Loader2 className="mr-1.5 size-4 animate-spin" aria-hidden />
              ) : (
                <Plus className="mr-1.5 size-4" aria-hidden />
              )}
              {importing ? t("knowledge.importing") : t("knowledge.upload")}
            </Button>
          </div>
        </WorkspaceSplitToolbar>

        <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4 pt-3">
          {error ? (
            <div className="flex items-start justify-between gap-2 rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
              <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-300">{error}</p>
              <button
                type="button"
                onClick={clearError}
                aria-label={t("knowledge.dismiss")}
                className="text-[length:var(--text-sm)] text-muted-foreground"
              >
                ×
              </button>
            </div>
          ) : null}

          <Card variant="solid" padding="sm">
            <div className="space-y-2.5">
              <p className="text-[length:var(--text-sm)] font-medium">{t("knowledge.toolsTitle")}</p>
              <p className="text-[length:var(--text-xs)] text-muted-foreground">
                {t("knowledge.toolsHint")}
              </p>
              {toolsLoading ? (
                <LoadingState label={t("knowledge.toolsLoading")} />
              ) : (
                <div className="space-y-2.5">
                  {tools.map((tool) => (
                    <ToolRow
                      key={tool.id}
                      tool={tool}
                      downloading={downloading[tool.id] ?? null}
                      onDownload={downloadTool}
                    />
                  ))}
                </div>
              )}
            </div>
          </Card>

          {loading ? (
            <LoadingState label={t("knowledge.loading")} />
          ) : documents.length === 0 ? (
            <div className="py-16 text-center">
              <FileText className="mx-auto size-8 text-muted-foreground/60" aria-hidden />
              <p className="mt-3 text-[length:var(--text-sm)] text-muted-foreground">
                {t("knowledge.empty")}
              </p>
              <p className="mt-1 text-[length:var(--text-xs)] text-muted-foreground/70">
                {t("knowledge.emptyHint")}
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {documents.map((doc) => (
                <DocumentRow
                  key={doc.id}
                  doc={doc}
                  deleting={deletingId === doc.id}
                  onDelete={remove}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </PageScaffold>
  );
}
