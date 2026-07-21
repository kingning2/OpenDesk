/**
 * 激活闸门锁图标动画。
 *
 * 负责：
 * - idle：静止锁
 * - busy：呼吸脉冲（校验中）
 * - success：锁对半裂开
 * - failure：锁加固（加厚 + 收紧）
 *
 * @author coisini
 * @created 2026-07-16
 */

import { cn, motion, spring } from "@desk/ui";

/**
 * 锁动画相位。
 *
 * @author coisini
 * @created 2026-07-16
 */
export type LicenseLockAnim =
  | "idle"
  | "busy"
  | "success"
  | "failure";

/**
 * 锁图标动画组件属性。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicenseLockGlyphProps {
  /** 当前动画相位。 */
  anim: LicenseLockAnim;
  /** 额外 class。 */
  className?: string;
}

/**
 * 可裂开 / 可加固的锁 SVG。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param props - 组件属性
 * @returns 动画锁节点
 */
export function LicenseLockGlyph({ anim, className }: LicenseLockGlyphProps) {
  const isBusy = anim === "busy";
  const isSuccess = anim === "success";
  const isFailure = anim === "failure";

  return (
    <motion.div
      className={cn("relative flex size-10 items-center justify-center", className)}
      animate={
        isBusy
          ? { scale: [1, 1.06, 1], opacity: [1, 0.85, 1] }
          : isFailure
            ? { scale: [1, 1.18, 1.08], rotate: [0, -4, 4, -2, 0] }
            : isSuccess
              ? { scale: [1, 1.08, 0.92] }
              : { scale: 1, opacity: 1, rotate: 0 }
      }
      transition={
        isBusy
          ? { duration: 1.1, repeat: Infinity, ease: "easeInOut" }
          : isFailure
            ? { duration: 0.55, ease: "easeOut" }
            : isSuccess
              ? { duration: 0.7, ease: "easeInOut" }
              : spring.snappy
      }
      aria-hidden
    >
      {/* 失败加固环 */}
      <motion.span
        className="pointer-events-none absolute inset-[-10px] rounded-full border-2 border-foreground/40"
        initial={false}
        animate={
          isFailure
            ? { opacity: [0, 1, 0.55], scale: [0.7, 1.05, 1] }
            : { opacity: 0, scale: 0.7 }
        }
        transition={{ duration: 0.55, ease: "easeOut" }}
      />

      <svg
        viewBox="0 0 40 40"
        className={cn(
          "size-full overflow-visible",
          isFailure ? "text-foreground" : "text-muted-foreground",
          isSuccess && "text-foreground",
        )}
        fill="none"
      >
        {/* 左半：弓环 + 锁体 */}
        <motion.g
          style={{ transformOrigin: "16px 22px" }}
          animate={
            isSuccess
              ? { x: -10, y: -2, rotate: -22, opacity: 0.35 }
              : { x: 0, y: 0, rotate: 0, opacity: 1 }
          }
          transition={{ duration: 0.65, ease: [0.22, 1, 0.36, 1] }}
        >
          <path
            d="M14 18.5V14.2c0-3.2 2.1-5.7 5-5.7"
            stroke="currentColor"
            strokeWidth={isFailure ? 2.6 : 1.7}
            strokeLinecap="round"
          />
          <path
            d="M11.5 18.5h8.5v14H13c-.8 0-1.5-.7-1.5-1.5v-12.5z"
            stroke="currentColor"
            strokeWidth={isFailure ? 2.6 : 1.7}
            strokeLinejoin="round"
          />
        </motion.g>

        {/* 右半 */}
        <motion.g
          style={{ transformOrigin: "24px 22px" }}
          animate={
            isSuccess
              ? { x: 10, y: -2, rotate: 22, opacity: 0.35 }
              : { x: 0, y: 0, rotate: 0, opacity: 1 }
          }
          transition={{ duration: 0.65, ease: [0.22, 1, 0.36, 1] }}
        >
          <path
            d="M26 18.5V14.2c0-3.2-2.1-5.7-5-5.7"
            stroke="currentColor"
            strokeWidth={isFailure ? 2.6 : 1.7}
            strokeLinecap="round"
          />
          <path
            d="M20 18.5h8.5v12.5c0 .8-.7 1.5-1.5 1.5H20V18.5z"
            stroke="currentColor"
            strokeWidth={isFailure ? 2.6 : 1.7}
            strokeLinejoin="round"
          />
        </motion.g>

        {/* 锁芯：成功时上移脱落，失败时放大加粗 */}
        <motion.circle
          cx="20"
          cy="25"
          r={isFailure ? 2.4 : 1.8}
          fill="currentColor"
          animate={
            isSuccess
              ? { cy: 12, opacity: 0, scale: 0.4 }
              : { cy: 25, opacity: 1, scale: 1 }
          }
          transition={{ duration: 0.55, ease: "easeOut" }}
        />

        {/* 成功裂痕闪光 */}
        <motion.line
          x1="20"
          y1="17"
          x2="20"
          y2="33"
          stroke="currentColor"
          strokeWidth={1.2}
          strokeLinecap="round"
          initial={false}
          animate={
            isSuccess
              ? { opacity: [0, 1, 0], pathLength: [0, 1, 1], scaleY: [0.2, 1.2, 1.4] }
              : { opacity: 0, pathLength: 0 }
          }
          transition={{ duration: 0.45, ease: "easeOut" }}
          style={{ transformOrigin: "20px 25px" }}
        />
      </svg>
    </motion.div>
  );
}
