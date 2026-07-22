/**
 * Script snippet library — two-column layout: category tree + card list + edit modal.
 * Design mirrors email-agent dashboard's 话术库 tab.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import { memo, useEffect, useMemo, useState } from "react";
import { ChevronRight, FileText, Folder, FolderOpen } from "@desk/ui/icons";
import {
  Button,
  Input,
  PageScaffold,
  WorkspaceSplit,
  WorkspaceSplitPane,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
  toast,
} from "@desk/ui";
import {
  type ScriptSnippet,
  workflowSnippetDelete,
  workflowSnippetList,
  workflowSnippetSave,
} from "@desk/platform";
import { useT } from "../../i18n";

interface FormState {
  id?: string;
  title: string;
  stage: string;
  trigger_text: string;
  description: string;
  from_stage: string;
  to_stage: string;
  body_text: string;
  category_l1: string;
  category_l2: string;
  needs_boss_input: boolean;
  boss_input_hint: string;
  sort_order: string;
  tags: string;
}

const EMPTY_FORM: FormState = {
  title: "",
  stage: "",
  trigger_text: "",
  description: "",
  from_stage: "",
  to_stage: "",
  body_text: "",
  category_l1: "",
  category_l2: "",
  needs_boss_input: false,
  boss_input_hint: "",
  sort_order: "0",
  tags: "",
};

function tagsJsonToStr(json: string): string {
  try {
    const arr = JSON.parse(json) as unknown;
    return Array.isArray(arr) ? arr.join(", ") : "";
  } catch {
    return "";
  }
}

function strToTagsJson(s: string): string {
  const tags = s
    .split(",")
    .map((t) => t.trim())
    .filter(Boolean);
  return JSON.stringify(tags);
}

function snippetToForm(s: ScriptSnippet): FormState {
  return {
    id: s.id,
    title: s.title ?? "",
    stage: s.stage ?? "",
    trigger_text: s.trigger_text ?? "",
    description: s.description ?? "",
    from_stage: s.from_stage ?? "",
    to_stage: s.to_stage ?? "",
    body_text: s.body_text ?? "",
    category_l1: s.category_l1 ?? "",
    category_l2: s.category_l2 ?? "",
    needs_boss_input: s.needs_boss_input ?? false,
    boss_input_hint: s.boss_input_hint ?? "",
    sort_order: String(s.sort_order ?? 0),
    tags: tagsJsonToStr(s.tags_json ?? "[]"),
  };
}

/**
 * Build a 2-level category tree from flat snippet list.
 */
function buildCategoryTree(snippets: ScriptSnippet[]) {
  const tree = new Map<string, Set<string>>();
  for (const s of snippets) {
    const l1 = s.category_l1 || "";
    if (!l1) continue;
    if (!tree.has(l1)) tree.set(l1, new Set());
    const l2 = s.category_l2 || "";
    if (l2) tree.get(l1)!.add(l2);
  }
  return tree;
}

function countByCategory(
  snippets: ScriptSnippet[],
  l1: string | null,
  l2: string | null,
): number {
  if (l1 === null) return snippets.length;
  return snippets.filter(
    (s) => s.category_l1 === l1 && (l2 === null || s.category_l2 === l2),
  ).length;
}

/**
 * Workflow script library page.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function WorkflowPage() {
  const t = useT();
  const [allSnippets, setAllSnippets] = useState<ScriptSnippet[]>([]);
  const [query, setQuery] = useState("");
  const [activeL1, setActiveL1] = useState<string | null>(null);
  const [activeL2, setActiveL2] = useState<string | null>(null);
  const [expandedL1, setExpandedL1] = useState<Record<string, boolean>>({});
  const [editing, setEditing] = useState<FormState | null>(null);

  async function loadAll() {
    try {
      const result = await workflowSnippetList({});
      setAllSnippets(result.items);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    let cancelled = false;
    void workflowSnippetList({})
      .then((result) => {
        if (!cancelled) {
          setAllSnippets(result.items);
        }
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

  const categoryTree = useMemo(() => buildCategoryTree(allSnippets), [allSnippets]);

  const filtered = useMemo(() => {
    return allSnippets.filter((s) => {
      if (activeL1 !== null && s.category_l1 !== activeL1) return false;
      if (activeL2 !== null && s.category_l2 !== activeL2) return false;
      if (query) {
        const kw = query.toLowerCase();
        return (
          (s.title ?? "").toLowerCase().includes(kw) ||
          (s.trigger_text ?? "").toLowerCase().includes(kw) ||
          (s.body_text ?? "").toLowerCase().includes(kw)
        );
      }
      return true;
    });
  }, [activeL1, activeL2, allSnippets, query]);

  function selectCategory(l1: string | null, l2: string | null) {
    setActiveL1(l1);
    setActiveL2(l2);
  }

  function toggleL1(l1: string) {
    setExpandedL1((prev) => ({ ...prev, [l1]: !(prev[l1] ?? true) }));
  }

  function openCreate() {
    setEditing({
      ...EMPTY_FORM,
      category_l1: activeL1 ?? "",
      category_l2: activeL2 ?? "",
    });
  }

  function openEdit(s: ScriptSnippet) {
    setEditing(snippetToForm(s));
  }

  async function handleCopy(text: string) {
    await navigator.clipboard.writeText(text);
    toast.success(t("workflow.copied"));
  }

  const folderLabel = useMemo(() => {
    if (!activeL1) return t("workflow.categoryAll");
    return activeL2 ? `${activeL1} / ${activeL2}` : activeL1;
  }, [activeL1, activeL2, t]);

  return (
    <PageScaffold fill containerPadding="none">
      <WorkspaceSplit className="min-h-0 flex-1 border-0">
        <WorkspaceSplitPane
          side="start"
          header={
            <div className="space-y-3 px-4 py-3">
              <WorkspaceSplitToolbar className="justify-between px-0 py-0">
                <WorkspaceSplitTitle>{t("workflow.title")}</WorkspaceSplitTitle>
              </WorkspaceSplitToolbar>
            </div>
          }
        >
          {/* Left: Category tree */}
          <div className="flex flex-col overflow-y-auto border-r bg-muted/30 py-2">
            {/* "全部" */}
            <button
              className={cn(
                "flex items-center justify-between px-4 py-1.5 text-left text-sm transition-colors hover:bg-muted",
                activeL1 === null && "bg-primary/10 font-medium text-primary",
              )}
              onClick={() => selectCategory(null, null)}
            >
              <span>📋 {t("workflow.categoryAll")}</span>
              <span className="rounded-full bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
                {allSnippets.length}
              </span>
            </button>

            {[...categoryTree.entries()].map(([l1, l2Set]) => (
              <div key={l1}>
                <div className="group flex items-center px-2">
                  <button
                    className={cn(
                      "mr-1 inline-flex size-5 items-center justify-center rounded-sm text-muted-foreground hover:bg-muted",
                    )}
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
                      {countByCategory(allSnippets, l1, null)}
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
                          {countByCategory(allSnippets, l1, l2)}
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </WorkspaceSplitPane>

        <WorkspaceSplitPane
          header={
            <WorkspaceSplitToolbar className="justify-between px-4 py-2.5">
              <Input
                placeholder={t("workflow.searchPlaceholder")}
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                className="h-7 w-48 text-xs"
              />
              <Button size="sm" onClick={openCreate}>
                {t("workflow.new")}
              </Button>
            </WorkspaceSplitToolbar>
          }
        >
          {/* Right: Card list */}
          <div className="flex flex-col overflow-hidden">
            {/* Sub-header */}
            <div className="flex items-center justify-between border-b px-4 py-2">
              <span className="text-xs text-muted-foreground">
                📁 {folderLabel} —{" "}
                {t("workflow.total").replace("{{count}}", String(filtered.length))}
              </span>
            </div>

            {/* Cards */}
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
                        className="cursor-pointer rounded-lg border bg-card p-3 transition-shadow hover:shadow-md"
                        onClick={() => openEdit(s)}
                      >
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <span className="font-mono font-bold text-primary">
                            #{s.source_id ?? s.id}
                          </span>
                          {s.category_l1 && (
                            <span>
                              📁 {s.category_l1}
                              {s.category_l2 ? ` / ${s.category_l2}` : ""}
                            </span>
                          )}
                        </div>
                        <div className="mt-1 truncate text-sm font-medium">
                          {(s.description || s.title || "(无描述)").slice(0, 80)}
                        </div>
                        <div className="mt-0.5 line-clamp-2 text-xs text-muted-foreground">
                          {(s.body_text || "").slice(0, 180)}
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
                          onClick={(e) => {
                            e.stopPropagation();
                            void handleCopy(s.body_text ?? "");
                          }}
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
        </WorkspaceSplitPane>
      </WorkspaceSplit>

      {editing ? (
        <SnippetEditModal
          initial={editing}
          onClose={() => setEditing(null)}
          onSaved={async () => {
            setEditing(null);
            await loadAll();
          }}
        />
      ) : null}
    </PageScaffold>
  );
}

/**
 * Snippet create/edit modal — owns draft form + saving so list keystrokes stay out of the page root.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const SnippetEditModal = memo(function SnippetEditModal({
  initial,
  onClose,
  onSaved,
}: {
  initial: FormState;
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const t = useT();
  const [form, setForm] = useState(initial);
  const [saving, setSaving] = useState(false);

  function updateField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }));
  }

  async function handleSave() {
    if (!form.title.trim()) return;
    setSaving(true);
    try {
      await workflowSnippetSave({
        id: form.id,
        title: form.title,
        stage: form.stage || undefined,
        trigger_text: form.trigger_text || undefined,
        description: form.description || undefined,
        from_stage: form.from_stage || undefined,
        to_stage: form.to_stage || undefined,
        body_text: form.body_text,
        category_l1: form.category_l1 || undefined,
        category_l2: form.category_l2 || undefined,
        needs_boss_input: form.needs_boss_input,
        boss_input_hint: form.boss_input_hint || undefined,
        sort_order: parseInt(form.sort_order) || 0,
        tags_json: strToTagsJson(form.tags),
      });
      toast.success(t("workflow.save"));
      await onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (!form.id) return;
    if (!window.confirm(t("workflow.deleteConfirm").replace("{{title}}", form.title))) return;
    try {
      await workflowSnippetDelete({ id: form.id });
      toast.success(t("workflow.delete"));
      await onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="flex max-h-[85vh] w-[720px] flex-col overflow-hidden rounded-xl border bg-card shadow-xl">
        <div className="flex items-center justify-between border-b px-5 py-3">
          <h4 className="text-sm font-semibold">
            {form.id ? t("workflow.form.editTitle") : t("workflow.form.createTitle")}
          </h4>
          <button className="text-muted-foreground hover:text-foreground" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="flex-1 overflow-y-auto px-5 py-4">
          <div className="grid grid-cols-2 gap-x-4 gap-y-3">
            <div className="col-span-2 flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.title")}
              </label>
              <Input value={form.title} onChange={(e) => updateField("title", e.target.value)} />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.categoryL1")}
              </label>
              <Input
                value={form.category_l1}
                onChange={(e) => updateField("category_l1", e.target.value)}
                placeholder="KOL"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.categoryL2")}
              </label>
              <Input
                value={form.category_l2}
                onChange={(e) => updateField("category_l2", e.target.value)}
                placeholder="初次联系"
              />
            </div>

            <div className="col-span-2 flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.description")}
              </label>
              <Input
                value={form.description}
                onChange={(e) => updateField("description", e.target.value)}
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.fromStage")}
              </label>
              <Input
                value={form.from_stage}
                onChange={(e) => updateField("from_stage", e.target.value)}
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.toStage")}
              </label>
              <Input
                value={form.to_stage}
                onChange={(e) => updateField("to_stage", e.target.value)}
              />
            </div>

            <div className="col-span-2 flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.tags")}
              </label>
              <Input
                value={form.tags}
                onChange={(e) => updateField("tags", e.target.value)}
                placeholder="tag1, tag2, tag3"
              />
            </div>

            <div className="col-span-2 flex flex-col gap-1">
              <label className="text-xs font-medium text-muted-foreground">
                {t("workflow.fields.body")}
              </label>
              <textarea
                value={form.body_text}
                onChange={(e) => updateField("body_text", e.target.value)}
                rows={8}
                className="resize-y rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              />
            </div>
          </div>
        </div>

        <div className="flex items-center justify-between border-t px-5 py-3">
          <div>
            {form.id ? (
              <Button
                size="sm"
                variant="outline"
                className="text-destructive hover:bg-destructive/10"
                onClick={() => void handleDelete()}
              >
                {t("workflow.delete")}
              </Button>
            ) : null}
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" onClick={onClose}>
              {t("workflow.cancel")}
            </Button>
            <Button size="sm" onClick={() => void handleSave()} disabled={saving}>
              {saving ? t("workflow.saving") : t("workflow.save")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
});
