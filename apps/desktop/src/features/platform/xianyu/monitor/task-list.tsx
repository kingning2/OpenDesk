import { Button } from "@desk/ui";
import type { MonitorTask } from "@desk/platform/ipc/xianyu-monitor";

export interface TaskListProps {
  tasks: MonitorTask[];
  selectedId: string;
  loading: boolean;
  onSelect: (task: MonitorTask) => void;
  onNew: () => void;
}

/** 监控任务列表（左栏）。 */
export function TaskList({ tasks, selectedId, loading, onSelect, onNew }: TaskListProps) {
  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-medium">监控任务</h2>
        <Button size="sm" variant="outline" onClick={onNew}>
          新建
        </Button>
      </div>
      <ul className="space-y-2">
        {tasks.map((task) => (
          <li key={task.id}>
            <button
              type="button"
              onClick={() => onSelect(task)}
              className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                selectedId === task.id
                  ? "border-primary bg-primary/10"
                  : "border-border bg-card hover:bg-muted/40"
              }`}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="font-medium">{task.name}</span>
                {task.isRunning ? (
                  <span className="text-xs text-primary">运行中</span>
                ) : task.enabled ? (
                  <span className="text-xs text-muted-foreground">启用</span>
                ) : (
                  <span className="text-xs text-muted-foreground">停用</span>
                )}
              </div>
              <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{task.intent}</p>
            </button>
          </li>
        ))}
      </ul>
      {!loading && tasks.length === 0 ? (
        <p className="text-sm text-muted-foreground">暂无监控任务，右侧创建第一个。</p>
      ) : null}
    </section>
  );
}
