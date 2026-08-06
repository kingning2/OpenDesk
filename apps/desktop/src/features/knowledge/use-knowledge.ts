/**
 * Knowledge hook — 文档列表、导入与删除。
 *
 * 导入由平台层弹原生文件选择窗口（`pickDocumentFile`）拿到磁盘路径，经 IPC 入队
 * `background_job`；解析在 `opendesk-worker` 进程执行。完成后主进程推
 * `knowledge:import/updated` 事件，此处订阅后刷新列表。
 *
 * @author coisini
 */

import { useCallback, useEffect, useState } from "react";
import { pickDocumentFile } from "@desk/platform";
import {
  knowledgeDocDelete,
  knowledgeDocImport,
  knowledgeDocList,
  knowledgeToolDownload,
  knowledgeToolStatus,
  listenKnowledgeImportUpdated,
  listenKnowledgeToolProgress,
  type KnowledgeDocument,
  type KnowledgeDownloadProgress,
  type KnowledgeToolStatus,
} from "@desk/platform/ipc/knowledge";

export type { KnowledgeDocument, KnowledgeToolStatus };

const toDisplayError = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

/**
 * 知识库文档管理 hook。
 *
 * @returns 文档列表、上传/删除操作与状态
 */
export function useKnowledge() {
  const [documents, setDocuments] = useState<KnowledgeDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [tools, setTools] = useState<KnowledgeToolStatus[]>([]);
  const [toolsLoading, setToolsLoading] = useState(true);
  const [downloading, setDownloading] = useState<Record<string, KnowledgeDownloadProgress | null>>(
    {},
  );

  const refreshTools = useCallback(async () => {
    try {
      const statuses = await knowledgeToolStatus();
      setTools(statuses);
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setToolsLoading(false);
    }
  }, []);

  // 初次挂载时加载列表与工具状态。用 .then 链避免在 effect 体内同步 setState。
  useEffect(() => {
    let cancelled = false;
    knowledgeDocList()
      .then((list) => {
        if (!cancelled) {
          setDocuments(list);
          setError("");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(toDisplayError(err));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    knowledgeToolStatus()
      .then((statuses) => {
        if (!cancelled) {
          setTools(statuses);
          setError("");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(toDisplayError(err));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setToolsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // 订阅下载进度事件，按工具更新进度；done/failed 后刷新工具状态。
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void listenKnowledgeToolProgress((payload) => {
      if (cancelled) {
        return;
      }
      setDownloading((prev) => ({ ...prev, [payload.tool]: payload }));
      if (payload.status === "done" || payload.status === "failed") {
        void refreshTools();
        setDownloading((prev) => ({ ...prev, [payload.tool]: null }));
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

  const refresh = useCallback(async () => {
    try {
      const list = await knowledgeDocList();
      setDocuments(list);
      setError("");
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  // 订阅知识库导入状态变化：worker 解析完成后主进程推事件，刷新文档列表。
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void listenKnowledgeImportUpdated(() => {
      if (!cancelled) {
        void refresh();
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
  }, [refresh]);

  const downloadTool = useCallback(
    async (toolId: string) => {
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
    },
    [],
  );

  // 弹文件选择窗口并入队导入；取消选择不动作。
  const pickFile = useCallback(async () => {
    setError("");
    const filePath = await pickDocumentFile();
    if (!filePath) {
      return;
    }
    setImporting(true);
    try {
      const response = await knowledgeDocImport({ file_path: filePath });
      if (!response.ok) {
        setError(response.error_message ?? "导入入队失败");
      }
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setImporting(false);
    }
  }, []);

  const remove = useCallback(
    async (documentId: string) => {
      setDeletingId(documentId);
      setError("");
      try {
        await knowledgeDocDelete({ document_id: documentId });
        await refresh();
      } catch (err) {
        setError(toDisplayError(err));
      } finally {
        setDeletingId(null);
      }
    },
    [refresh],
  );

  const clearError = useCallback(() => setError(""), []);

  return {
    documents,
    loading,
    importing,
    deletingId,
    error,
    pickFile,
    remove,
    clearError,
    refresh,
    tools,
    toolsLoading,
    downloading,
    downloadTool,
    refreshTools,
  };
}
