/**
 * Workflow template detail — renders a parsed workflow canvas: stage flow,
 * routing rules, and extra canvas metadata.
 *
 * @author coisini
 * @created 2026-08-07
 */

import { useEffect, useMemo, useState } from "react";
import { Loader2 } from "@desk/ui/icons";
import { cn, toast } from "@desk/ui";
import {
  type WorkflowRule,
  type WorkflowTemplate,
  workflowRuleList,
  workflowTemplateGet,
} from "@desk/platform";
import { useT } from "../../i18n";
import { parseCanvas, type WorkflowCanvas } from "./workflow-types";

function jsonArray(json: string | undefined): unknown[] {
  if (!json) return [];
  try {
    const parsed = JSON.parse(json) as unknown;
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function StageScripts({
  scripts,
  conds,
}: {
  scripts?: string[];
  conds?: string[];
}) {
  if (!scripts?.length && !conds?.length) return null;
  return (
    <div className="mt-2 space-y-1">
      {scripts?.map((s, i) => (
        <pre
          key={i}
          className="whitespace-pre-wrap rounded bg-muted/60 px-2 py-1.5 text-[11px] leading-relaxed text-foreground/80"
        >
          {s}
        </pre>
      ))}
      {conds?.length ? (
        <p className="text-[11px] text-muted-foreground">
          {conds.join(" · ")}
        </p>
      ) : null}
    </div>
  );
}

/**
 * 工作流模板详情。
 *
 * @author coisini
 * @created 2026-08-07
 */
export function WorkflowTemplateDetail({
  templateId,
}: {
  templateId: string;
}) {
  const t = useT();
  const [template, setTemplate] = useState<WorkflowTemplate | null>(null);
  const [rules, setRules] = useState<WorkflowRule[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void Promise.all([workflowTemplateGet(templateId), workflowRuleList()])
      .then(([tpl, ruleResult]) => {
        if (cancelled) return;
        setTemplate(tpl);
        setRules(ruleResult.items);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          setError(message);
          toast.error(message);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [templateId]);

  const canvas: WorkflowCanvas | null = useMemo(() => {
    if (!template) return null;
    return parseCanvas(template.canvas_json);
  }, [template]);

  if (error) {
    return (
      <div className="flex flex-1 items-center justify-center p-8">
        <p className="text-sm text-muted-foreground">{error}</p>
      </div>
    );
  }

  if (!template || !canvas) {
    return (
      <div className="flex flex-1 items-center justify-center gap-2 p-8 text-sm text-muted-foreground">
        <Loader2 className="size-4 animate-spin" />
        {t("loading")}
      </div>
    );
  }

  const isWhatsApp = template.template_type === "whatsapp";
  const typeLabel = isWhatsApp ? t("workflow.typeWhatsapp") : t("workflow.typeEmail");
  const emailTypes = jsonArray(canvas.emailTypes as string | undefined);
  const archiveStates = jsonArray(canvas.archiveStates as string | undefined);
  const connections = jsonArray(canvas.connections as string | undefined);

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="p-4">
        {/* Header */}
        <div className="flex flex-wrap items-center gap-2 border-b border-border/70 pb-3">
          <h3 className="text-base font-semibold text-foreground">{template.name}</h3>
          <span
            className={cn(
              "rounded px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide",
              isWhatsApp
                ? "bg-emerald-500/15 text-emerald-600"
                : "bg-sky-500/15 text-sky-600",
            )}
          >
            {typeLabel}
          </span>
          {typeof template.binding_count === "number" && template.binding_count > 0 ? (
            <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground">
              {t("workflow.bindingCount", { count: template.binding_count })}
            </span>
          ) : null}
          {canvas.updated ? (
            <span className="ml-auto text-xs text-muted-foreground">
              {t("workflow.templateUpdated", { time: String(canvas.updated) })}
            </span>
          ) : null}
        </div>

        {/* Stage flow */}
        <div className="mt-4">
          <h4 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {t("workflow.stageSection")}
          </h4>
          <div className="flex flex-col gap-2">
            {canvas.stages.map((stage, index) => (
              <details key={stage.id} className="group rounded-lg border bg-card">
                <summary className="flex cursor-pointer list-none items-center gap-2 px-3 py-2.5">
                  <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 font-mono text-[10px] font-bold text-primary">
                    {index + 1}
                  </span>
                  <span className="text-sm font-medium">{stage.name}</span>
                  {stage.aiLevel ? (
                    <span className="rounded bg-violet-500/15 px-1.5 py-0.5 text-[10px] text-violet-600">
                      {stage.aiLevel}
                    </span>
                  ) : null}
                  {stage.id ? (
                    <span className="font-mono text-[10px] text-muted-foreground">
                      {stage.id}
                    </span>
                  ) : null}
                  <ChevronIndicator />
                </summary>
                <div className="border-t border-border/60 px-3 py-2.5">
                  {stage.note ? (
                    <p className="whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">
                      {stage.note}
                    </p>
                  ) : null}
                  <StageScripts scripts={stage.scripts} conds={stage.scriptConds} />
                </div>
              </details>
            ))}
          </div>
        </div>

        {/* Routing rules */}
        <div className="mt-6">
          <h4 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {t("workflow.routeRules")} ({rules.length})
          </h4>
          {rules.length === 0 ? (
            <p className="py-4 text-center text-sm text-muted-foreground">
              {t("workflow.empty")}
            </p>
          ) : (
            <div className="flex flex-col gap-2">
              {rules.map((rule) => {
                const fromStages = jsonArray(rule.from_stages_json) as string[];
                const keywords = jsonArray(rule.trigger_keywords_json) as string[];
                const tags = jsonArray(rule.trigger_tags_json) as string[];
                return (
                  <div key={rule.id} className="rounded-lg border bg-card px-3 py-2.5">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-sm font-medium">{rule.name}</span>
                      {rule.auto_reply ? (
                        <span className="rounded bg-primary/10 px-1.5 py-0.5 text-[10px] text-primary">
                          {t("workflow.routeAutoReply")}
                        </span>
                      ) : null}
                      {rule.auto_advance ? (
                        <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 text-[10px] text-emerald-600">
                          {t("workflow.routeAutoAdvance")}
                        </span>
                      ) : null}
                    </div>
                    <div className="mt-1.5 flex flex-wrap items-center gap-1 text-xs text-muted-foreground">
                      <span>{t("workflow.routeFrom")}</span>
                      {fromStages.map((s) => (
                        <span
                          key={s}
                          className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]"
                        >
                          {s}
                        </span>
                      ))}
                      <span className="mx-1">→</span>
                      <span className="font-mono text-foreground/80">{rule.to_stage}</span>
                    </div>
                    {keywords.length > 0 && (
                      <p className="mt-1.5 text-[11px] leading-relaxed text-muted-foreground">
                        {keywords.join(" · ")}
                      </p>
                    )}
                    {tags.length > 0 && (
                      <div className="mt-1 flex flex-wrap gap-1">
                        {tags.map((tag) => (
                          <span
                            key={tag}
                            className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Extra canvas metadata (collapsed) */}
        {emailTypes.length > 0 || archiveStates.length > 0 || connections.length > 0 ? (
          <div className="mt-6 space-y-2">
            {emailTypes.length > 0 && (
              <CanvasSection
                title={t("workflow.emailTypes")}
                items={emailTypes.map((node) => {
                  const n = node as { name?: string; note?: string; desc?: string; id?: string };
                  return {
                    key: n.id ?? String(node),
                    title: n.name ?? String(node),
                    detail: [n.desc, n.note].filter(Boolean).join(" "),
                  };
                })}
              />
            )}
            {archiveStates.length > 0 && (
              <CanvasSection
                title={t("workflow.archiveStates")}
                items={archiveStates.map((node) => {
                  const n = node as { label?: string; scope?: string; id?: string };
                  return {
                    key: n.id ?? String(node),
                    title: n.label ?? String(node),
                    detail: n.scope ?? "",
                  };
                })}
              />
            )}
            {connections.length > 0 && (
              <CanvasSection
                title={t("workflow.connections")}
                items={connections.map((node) => {
                  const n = node as { id?: string; from?: string; to?: string };
                  return {
                    key: n.id ?? String(node),
                    title: `${n.from ?? "?"} → ${n.to ?? "?"}`,
                    detail: "",
                  };
                })}
              />
            )}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function ChevronIndicator() {
  return (
    <span className="ml-auto inline-block text-xs text-muted-foreground transition-transform group-open:rotate-180">
      ▾
    </span>
  );
}

function CanvasSection({
  title,
  items,
}: {
  title: string;
  items: Array<{ key: string; title: string; detail: string }>;
}) {
  return (
    <details className="group rounded-lg border bg-card">
      <summary className="flex cursor-pointer list-none items-center gap-2 px-3 py-2.5 text-sm font-medium">
        {title}
        <span className="rounded-full bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
          {items.length}
        </span>
        <ChevronIndicator />
      </summary>
      <div className="grid grid-cols-1 gap-1 border-t border-border/60 px-3 py-2.5 sm:grid-cols-2">
        {items.map((item) => (
          <div key={item.key} className="text-xs">
            <span className="font-medium text-foreground/90">{item.title}</span>
            {item.detail ? (
              <p className="line-clamp-2 text-muted-foreground">{item.detail}</p>
            ) : null}
          </div>
        ))}
      </div>
    </details>
  );
}
