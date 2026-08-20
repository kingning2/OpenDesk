/**
 * 环境聚光灯背景 — 改编自 Aceternity UI Spotlight。
 *
 * @see https://ui.aceternity.com/components/spotlight-new
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { motion } from "motion/react";

import { cn } from "../../lib/cn";

/**
 * 聚光灯背景属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface AmbientSpotlightProps {
  className?: string;
  /** 主动画周期（秒）。 */
  duration?: number;
  /** 水平摆动幅度（px）。 */
  xOffset?: number;
}

const GRADIENT_A =
  "radial-gradient(68.54% 68.72% at 55.02% 31.46%, oklch(0.62 0.19 285 / 0.12) 0, oklch(0.55 0.2 280 / 0.04) 50%, transparent 80%)";
const GRADIENT_B =
  "radial-gradient(50% 50% at 50% 50%, oklch(0.62 0.19 285 / 0.08) 0, oklch(0.55 0.2 280 / 0.03) 80%, transparent 100%)";
const GRADIENT_C =
  "radial-gradient(50% 50% at 50% 50%, oklch(0.7 0.15 300 / 0.06) 0, oklch(0.55 0.2 280 / 0.02) 80%, transparent 100%)";

/**
 * 工作区环境聚光灯 — 用于首页等入口页背景，pointer-events-none。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function AmbientSpotlight({
  className,
  duration = 7,
  xOffset = 100,
}: AmbientSpotlightProps) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 1.5 }}
      className={cn("pointer-events-none absolute inset-0 overflow-hidden", className)}
      aria-hidden
    >
      <motion.div
        animate={{ x: [0, xOffset, 0] }}
        transition={{ duration, repeat: Infinity, repeatType: "reverse", ease: "easeInOut" }}
        className="pointer-events-none absolute top-0 left-0 h-full w-full"
      >
        <div
          style={{
            transform: "translateY(-350px) rotate(-45deg)",
            background: GRADIENT_A,
            width: 560,
            height: 1380,
          }}
          className="absolute top-0 left-0"
        />
        <div
          style={{
            transform: "rotate(-45deg) translate(5%, -50%)",
            background: GRADIENT_B,
            width: 240,
            height: 1380,
          }}
          className="absolute top-0 left-0 origin-top-left"
        />
        <div
          style={{
            transform: "rotate(-45deg) translate(-180%, -70%)",
            background: GRADIENT_C,
            width: 240,
            height: 1380,
          }}
          className="absolute top-0 left-0 origin-top-left"
        />
      </motion.div>

      <motion.div
        animate={{ x: [0, -xOffset, 0] }}
        transition={{ duration, repeat: Infinity, repeatType: "reverse", ease: "easeInOut" }}
        className="pointer-events-none absolute top-0 right-0 h-full w-full"
      >
        <div
          style={{
            transform: "translateY(-350px) rotate(45deg)",
            background: GRADIENT_A,
            width: 560,
            height: 1380,
          }}
          className="absolute top-0 right-0"
        />
        <div
          style={{
            transform: "rotate(45deg) translate(-5%, -50%)",
            background: GRADIENT_B,
            width: 240,
            height: 1380,
          }}
          className="absolute top-0 right-0 origin-top-right"
        />
        <div
          style={{
            transform: "rotate(45deg) translate(180%, -70%)",
            background: GRADIENT_C,
            width: 240,
            height: 1380,
          }}
          className="absolute top-0 right-0 origin-top-right"
        />
      </motion.div>
    </motion.div>
  );
}
