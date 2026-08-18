/**
 * 工作区布局策略 — 决定最外层 PageScaffold 是否开启滚动。
 *
 * 桌面壳专用；Web 端若有不同布局规则，在对应 app 内单独定义。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

/**
 * 判断路径是否需占满高度并由内部自管滚动（关闭外层 ScrollArea）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param path - 工作区路径
 * @returns 需 fill 布局且关闭外层滚动时为 `true`
 */
export function needsFillLayout(path: string): boolean {
  if (path.endsWith("/tutorial")) {
    return true;
  }
  return false;
}
