/**
 * Workflow feature page — template list + detail / script library switch.
 * Replaces the former 话术库 (script_snippet) page; data comes from the
 * email-agent workflow import.
 *
 * @author coisini
 * @created 2026-08-07
 */

import { useEffect, useState } from "react";
import { FileText, Loader2, Mail } from "@desk/ui/icons";
import {
  Button,
  PageScaffold,
  WorkspaceSplit,
  WorkspaceSplitPane,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
  toast,
} from "@desk/ui";
import { type WorkflowTemplate, workflowTemplateList } from "@desk/platform";
import { useT } from "../../i18n";
import { WorkflowTemplateDetail } from "./workflow-template-detail";
import { WorkflowScriptLibrary } from "./workflow-script-library";

type ActiveTab = "template" | "scripts";

/**
 * Workflow page.
 *
 * @author coisini
 * @created 2026-08-07
 */
export function WorkflowPage() {
  const t = useT();
  const [templates, setTemplates] = useState<WorkflowTemplate[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<ActiveTab>("template");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    void workflowTemplateList()
      .then((result) => {
        if (cancelled) return;
        setTemplates(result.items);
        if (result.items.length > 0 && selectedId === null) {
          setSelectedId(result.items[0]!.id);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function isWhatsApp(template: WorkflowTemplate) {
    return template.template_type === "whatsapp";
  }

  return (
    <PageScaffold fill containerPadding="none">
      <WorkspaceSplit className="border-0">
        {/* Start: template list + tab switch */}
        <WorkspaceSplitPane
          side="start"
          scroll={false}
          header={
            <WorkspaceSplitToolbar className="justify-between">
              <WorkspaceSplitTitle>{t("workflow.title")}</WorkspaceSplitTitle>
            </WorkspaceSplitToolbar>
          }
        >
          <div className="flex min-h-0 flex-1 flex-col">
            <div className="flex-1 overflow-y-auto">
              {loading ? (
                <div className="flex items-center justify-center gap-2 py-10 text-sm text-muted-foreground">
                  <Loader2 className="size-4 animate-spin" />
                  {t("loading")}
                </div>
              ) : templates.length === 0 ? (
                <p className="px-4 py-10 text-center text-sm text-muted-foreground">
                  {t("workflow.noTemplate")}
                </p>
              ) : (
                templates.map((template) => (
                  <button
                    key={template.id}
                    type="button"
                    className={cn(
                      "flex w-full items-center gap-2 border-b border-border/50 px-4 py-2.5 text-left transition-colors hover:bg-muted/40",
                      selectedId === template.id && "bg-primary/10",
                    )}
                    onClick={() => {
                      setSelectedId(template.id);
                      setActiveTab("template");
                    }}
                  >
                    {isWhatsApp(template) ? (
                      <Mail className="size-4 shrink-0 text-emerald-500" />
                    ) : (
                      <Mail className="size-4 shrink-0 text-sky-500" />
                    )}
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-medium text-foreground">
                        {template.name}
                      </div>
                      <div className="mt-0.5 flex items-center gap-1.5 text-[11px] text-muted-foreground">
                        <span
                          className={cn(
                            "rounded px-1 py-px font-medium uppercase tracking-wide",
                            isWhatsApp(template)
                              ? "bg-emerald-500/15 text-emerald-600"
                              : "bg-sky-500/15 text-sky-600",
                          )}
                        >
                          {isWhatsApp(template)
                            ? t("workflow.typeWhatsapp")
                            : t("workflow.typeEmail")}
                        </span>
                        {typeof template.binding_count === "number" &&
                        template.binding_count > 0 ? (
                          <span className="tabular-nums">
                            {t("workflow.bindingCount", { count: template.binding_count })}
                          </span>
                        ) : null}
                      </div>
                    </div>
                  </button>
                ))
              )}
            </div>

            {/* Bottom tab switch */}
            <div className="grid shrink-0 grid-cols-2 gap-1 border-t border-border/70 p-2">
              <Button
                size="sm"
                variant={activeTab === "template" ? "default" : "ghost"}
                className="h-8"
                onClick={() => setActiveTab("template")}
              >
                {t("workflow.tabsTemplate")}
              </Button>
              <Button
                size="sm"
                variant={activeTab === "scripts" ? "default" : "ghost"}
                className="h-8"
                onClick={() => setActiveTab("scripts")}
              >
                {t("workflow.tabsScripts")}
              </Button>
            </div>
          </div>
        </WorkspaceSplitPane>

        {/* Main: template detail or script library */}
        <WorkspaceSplitPane side="main" scroll={false}>
          {activeTab === "scripts" ? (
            <WorkflowScriptLibrary />
          ) : selectedId ? (
            <WorkflowTemplateDetail key={selectedId} templateId={selectedId} />
          ) : (
            <div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
              <FileText className="size-8 text-muted-foreground/40" />
              <p className="text-sm text-muted-foreground">{t("workflow.noTemplate")}</p>
            </div>
          )}
        </WorkspaceSplitPane>
      </WorkspaceSplit>
    </PageScaffold>
  );
}
