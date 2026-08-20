/**
 * 带 Aceternity 光晕边框的列表卡片容器。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import * as React from "react";

import { GlowingEffect } from "../effects/glowing-effect";
import { cn } from "../../lib/cn";
import { useReducedMotion } from "../../motion";

/**
 * 光晕卡片属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export type PageGlowCardProps = React.HTMLAttributes<HTMLDivElement>;

/**
 * 列表页交互卡片外壳 — 内置 {@link GlowingEffect}，减少 motion 时退化为普通边框。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function PageGlowCard({ className, children, ...props }: PageGlowCardProps) {
  const reducedMotion = useReducedMotion();

  return (
    <div className={cn("relative rounded-2xl", className)} {...props}>
      {!reducedMotion ? (
        <GlowingEffect
          disabled={false}
          spread={36}
          proximity={72}
          inactiveZone={0.15}
          borderWidth={1}
        />
      ) : null}
      {children}
    </div>
  );
}
