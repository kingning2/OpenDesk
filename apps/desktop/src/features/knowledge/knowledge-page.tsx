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
import { useKnowledge, type KnowledgeDocument } from "./use-knowledge";

/** 文档来源类型显示名。 */
function sourceTypeLabel(type: string): string {
  return type.toUpperCase();
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
