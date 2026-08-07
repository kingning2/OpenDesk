/**
 * Read-only script library — two-column layout: category tree + card list.
 * Design mirrors email-agent dashboard's 话术库 tab.
 *
 * @author coisini
 * @created 2026-08-07
 */

import { useEffect, useMemo, useState } from "react";
import { ChevronRight, FileText, Folder, FolderOpen, Search } from "@desk/ui/icons";
import { Input, ScrollArea, cn, toast } from "@desk/ui";
import { type WorkflowScript, workflowScriptList } from "@desk/platform";
import { copyToClipboard } from "@desk/utils";
import { useT } from "../../i18n";

/**
 * Build a 2-level category tree from flat script list.
 */
function buildCategoryTree(scripts: WorkflowScript[]) {
  const tree = new Map<string, Set<string>>();
  for (const s of scripts) {
    const l1 = s.category_l1 || "";
    if (!l1) continue;
    if (!tree.has(l1)) tree.set(l1, new Set());
    const l2 = s.category_l2 || "";
    if (l2) tree.get(l1)!.add(l2);
  }
  return tree;
}

function countByCategory(
  scripts: WorkflowScript[],
  l1: string | null,
  l2: string | null,
): number {
  if (l1 === null) return scripts.length;
  return scripts.filter(
    (s) => s.category_l1 === l1 && (l2 === null || s.category_l2 === l2),
  ).length;
}

/**
 * 话术库 — email-agent 话术数据只读浏览。
 *
 * @author coisini
 * @created 2026-08-07
 */
export function WorkflowScriptLibrary() {
  const t = useT();
  const [allScripts, setAllScripts] = useState<WorkflowScript[]>([]);
  const [query, setQuery] = useState("");
  const [activeL1, setActiveL1] = useState<string | null>(null);
  const [activeL2, setActiveL2] = useState<string | null>(null);
  const [expandedL1, setExpandedL1] = useState<Record<string, boolean>>({});

  useEffect(() => {
    let cancelled = false;
    void workflowScriptList({})
      .then((result) => {
        if (!cancelled) setAllScripts(result.items);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const categoryTree = useMemo(() => buildCategoryTree(allScripts), [allScripts]);

  const filtered = useMemo(() => {
    return allScripts.filter((s) => {
      if (activeL1 !== null && s.category_l1 !== activeL1) return false;
      if (activeL2 !== null && s.category_l2 !== activeL2) return false;
      if (query) {
        const kw = query.toLowerCase();
        return (
          (s.description ?? "").toLowerCase().includes(kw) ||
          (s.trigger_text ?? "").toLowerCase().includes(kw) ||
          s.content.toLowerCase().includes(kw)
        );
      }
      return true;
    });
  }, [activeL1, activeL2, allScripts, query]);

  function selectCategory(l1: string | null, l2: string | null) {
    setActiveL1(l1);
    setActiveL2(l2);
  }

  function toggleL1(l1: string) {
    setExpandedL1((prev) => ({ ...prev, [l1]: !(prev[l1] ?? true) }));
  }

  async function handleCopy(text: string) {
    await copyToClipboard(text);
    toast.success(t("workflow.copied"));
  }

  const folderLabel = useMemo(() => {
    if (!activeL1) return t("workflow.categoryAll");
    return activeL2 ? `${activeL1} / ${activeL2}` : activeL1;
  }, [activeL1, activeL2, t]);

  return (
    <div className="flex min-h-0 flex-1 overflow-hidden">
      {/* Left: Category tree */}
      <div className="flex w-[240px] shrink-0 flex-col border-r border-border/70 bg-muted/30">
        <div className="flex items-center gap-2 border-b border-border/70 px-3 py-2">
          <Search className="size-3.5 text-muted-foreground" />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("workflow.searchPlaceholder")}
            className="h-7 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
          />
        </div>
        <ScrollArea className="min-h-0 flex-1">
          <div className="py-2">
            <button
              className={cn(
                "flex w-full items-center justify-between px-4 py-1.5 text-left text-sm transition-colors hover:bg-muted",
                activeL1 === null && "bg-primary/10 font-medium text-primary",
              )}
              onClick={() => selectCategory(null, null)}
            >
              <span>{t("workflow.categoryAll")}</span>
              <span className="rounded-full bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
                {allScripts.length}
              </span>
            </button>

            {[...categoryTree.entries()].map(([l1, l2Set]) => (
              <div key={l1}>
                <div className="group flex items-center px-2">
                  <button
                    className="mr-1 inline-flex size-5 items-center justify-center rounded-sm text-muted-foreground hover:bg-muted"
                    onClick={() => toggleL1(l1)}
                    aria-label={`toggle-${l1}`}
                  >
                    <ChevronRight
                      className={cn(
                        "size-3.5 transition-transform",
                        (expandedL1[l1] ?? true) && "rotate-90",
                      )}
                    />
                  </button>
                  <button
                    className={cn(
                      "flex flex-1 items-center justify-between rounded-md px-2 py-1.5 text-left text-sm font-medium transition-colors hover:bg-muted",
                      activeL1 === l1 && activeL2 === null && "bg-primary/10 text-primary",
                    )}
                    onClick={() => selectCategory(l1, null)}
                  >
                    <span className="inline-flex items-center gap-1.5">
                      {(expandedL1[l1] ?? true) ? (
                        <FolderOpen className="size-4 text-sky-500" />
                      ) : (
                        <Folder className="size-4 text-sky-500" />
                      )}
                      {l1}
                    </span>
                    <span className="rounded-full bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
                      {countByCategory(allScripts, l1, null)}
                    </span>
                  </button>
                </div>
                {(expandedL1[l1] ?? true) && (
                  <div className="ml-5 border-l border-border/60 pl-2">
                    {[...l2Set].map((l2) => (
                      <button
                        key={l2}
                        className={cn(
                          "relative flex w-full items-center justify-between rounded-md py-1.5 pl-2 pr-3 text-left text-xs transition-colors hover:bg-muted",
                          "before:absolute before:-left-2 before:top-1/2 before:h-px before:w-2 before:bg-border/70 before:content-['']",
                          activeL1 === l1 && activeL2 === l2 && "bg-primary/10 text-primary",
                        )}
                        onClick={() => selectCategory(l1, l2)}
                      >
                        <span className="inline-flex items-center gap-1.5">
                          <FileText className="size-3.5 text-slate-500" />
                          {l2}
                        </span>
                        <span className="rounded-full bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
                          {countByCategory(allScripts, l1, l2)}
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* Right: Card list */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between border-b border-border/70 px-4 py-2">
          <span className="text-xs text-muted-foreground">
            {folderLabel} — {t("workflow.total", { count: filtered.length })}
          </span>
        </div>

        <div className="flex-1 overflow-y-auto p-3">
          {filtered.length === 0 ? (
            <p className="py-12 text-center text-sm text-muted-foreground">
              {t("workflow.empty")}
            </p>
          ) : (
            <div className="flex flex-col gap-2">
              {filtered.map((s) => {
                const tags = [
                  s.category_l1,
                  s.category_l2,
                  ...(JSON.parse(s.tags_json || "[]") as string[]),
                  s.stage,
                ].filter(Boolean);
                return (
                  <div
                    key={s.id}
                    className="rounded-lg border bg-card p-3 transition-shadow hover:shadow-md"
                  >
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span className="font-mono font-bold text-primary">#{s.id}</span>
                      {s.category_l1 && (
                        <span>
                          {s.category_l1}
                          {s.category_l2 ? ` / ${s.category_l2}` : ""}
                        </span>
                      )}
                      {s.from_stage && s.to_stage ? (
                        <span>
                          {s.from_stage} → {s.to_stage}
                        </span>
                      ) : null}
                    </div>
                    <div className="mt-1 truncate text-sm font-medium">
                      {(s.description || s.trigger_text || "(无标题)").slice(0, 80)}
                    </div>
                    <div className="mt-0.5 line-clamp-3 whitespace-pre-wrap text-xs text-muted-foreground">
                      {s.content.slice(0, 240)}
                    </div>
                    {tags.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {tags.map((tag, i) => (
                          <span
                            key={i}
                            className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                    <button
                      className="mt-2 text-xs text-primary hover:underline"
                      onClick={() => void handleCopy(s.content)}
                    >
                      {t("workflow.copy")}
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
