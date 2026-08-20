/**
 * 鼠标跟随边框光晕 — 改编自 Aceternity UI Glowing Effect。
 *
 * @see https://ui.aceternity.com/components/glowing-effect
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { memo, useCallback, useEffect, useRef, type CSSProperties } from "react";
import { animate } from "motion/react";

import { cn } from "../../lib/cn";

/**
 * 光晕效果属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface GlowingEffectProps {
  /** 光晕模糊像素。 */
  blur?: number;
  /** 中心禁用区半径倍率（0–1）。 */
  inactiveZone?: number;
  /** 元素外扩仍激活的距离（px）。 */
  proximity?: number;
  /** 光晕角度 spread（deg）。 */
  spread?: number;
  /** 配色：`default` 主题紫，`white` 单色。 */
  variant?: "default" | "white";
  /** 常亮，不依赖 hover。 */
  glow?: boolean;
  className?: string;
  /** 为 true 时关闭交互光晕。 */
  disabled?: boolean;
  /** 光晕旋转动画时长（秒）。 */
  movementDuration?: number;
  /** 边框宽度（px）。 */
  borderWidth?: number;
}

const PRIMARY_GRADIENT = `radial-gradient(circle, oklch(0.62 0.19 285 / 0.55) 10%, oklch(0.62 0.19 285 / 0) 20%),
radial-gradient(circle at 40% 40%, oklch(0.55 0.2 280 / 0.45) 5%, transparent 15%),
radial-gradient(circle at 60% 60%, oklch(0.7 0.15 300 / 0.4) 10%, transparent 20%),
radial-gradient(circle at 40% 60%, oklch(0.5 0.12 260 / 0.4) 10%, transparent 20%),
repeating-conic-gradient(
  from 236.84deg at 50% 50%,
  oklch(0.62 0.19 285) 0%,
  oklch(0.55 0.2 280) calc(25% / var(--repeating-conic-gradient-times)),
  oklch(0.7 0.15 300) calc(50% / var(--repeating-conic-gradient-times)),
  oklch(0.5 0.12 260) calc(75% / var(--repeating-conic-gradient-times)),
  oklch(0.62 0.19 285) calc(100% / var(--repeating-conic-gradient-times))
)`;

/**
 * 鼠标跟随边框光晕。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export const GlowingEffect = memo(function GlowingEffect({
  blur = 0,
  inactiveZone = 0.7,
  proximity = 0,
  spread = 20,
  variant = "default",
  glow = false,
  className,
  movementDuration = 2,
  borderWidth = 1,
  disabled = true,
}: GlowingEffectProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const lastPosition = useRef({ x: 0, y: 0 });
  const animationFrameRef = useRef(0);

  const handleMove = useCallback(
    (e?: MouseEvent | { x: number; y: number }) => {
      const element = containerRef.current;
      if (!element) {
        return;
      }

      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      animationFrameRef.current = requestAnimationFrame(() => {
        if (!containerRef.current) {
          return;
        }

        const { left, top, width, height } = containerRef.current.getBoundingClientRect();
        const mouseX = e?.x ?? lastPosition.current.x;
        const mouseY = e?.y ?? lastPosition.current.y;

        if (e) {
          lastPosition.current = { x: mouseX, y: mouseY };
        }

        const center = [left + width * 0.5, top + height * 0.5];
        const distanceFromCenter = Math.hypot(mouseX - center[0], mouseY - center[1]);
        const inactiveRadius = 0.5 * Math.min(width, height) * inactiveZone;

        if (distanceFromCenter < inactiveRadius) {
          containerRef.current.style.setProperty("--active", "0");
          return;
        }

        const isActive =
          mouseX > left - proximity &&
          mouseX < left + width + proximity &&
          mouseY > top - proximity &&
          mouseY < top + height + proximity;

        containerRef.current.style.setProperty("--active", isActive ? "1" : "0");
        if (!isActive) {
          return;
        }

        const currentAngle =
          parseFloat(containerRef.current.style.getPropertyValue("--start")) || 0;
        const targetAngle =
          (180 * Math.atan2(mouseY - center[1], mouseX - center[0])) / Math.PI + 90;
        const angleDiff = ((targetAngle - currentAngle + 180) % 360) - 180;
        const newAngle = currentAngle + angleDiff;

        animate(currentAngle, newAngle, {
          duration: movementDuration,
          ease: [0.16, 1, 0.3, 1],
          onUpdate: (value) => {
            containerRef.current?.style.setProperty("--start", String(value));
          },
        });
      });
    },
    [inactiveZone, proximity, movementDuration],
  );

  useEffect(() => {
    if (disabled) {
      return;
    }

    const handleScroll = () => handleMove();
    const handlePointerMove = (event: PointerEvent) => handleMove(event);

    window.addEventListener("scroll", handleScroll, { passive: true });
    document.body.addEventListener("pointermove", handlePointerMove, { passive: true });

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      window.removeEventListener("scroll", handleScroll);
      document.body.removeEventListener("pointermove", handlePointerMove);
    };
  }, [handleMove, disabled]);

  const gradient =
    variant === "white"
      ? `repeating-conic-gradient(
          from 236.84deg at 50% 50%,
          var(--color-foreground) calc(0% / var(--repeating-conic-gradient-times)),
          transparent calc(25% / var(--repeating-conic-gradient-times))
        )`
      : PRIMARY_GRADIENT;

  const effectStyle = {
    "--blur": `${blur}px`,
    "--spread": spread,
    "--start": "0",
    "--active": "0",
    "--glowingeffect-border-width": `${borderWidth}px`,
    "--repeating-conic-gradient-times": "5",
    "--gradient": gradient,
  } as CSSProperties;

  return (
    <>
      <div
        className={cn(
          "pointer-events-none absolute -inset-px hidden rounded-[inherit] border opacity-0 transition-opacity",
          glow && "opacity-100",
          variant === "white" && "border-border",
          disabled && "!block",
        )}
      />
      <div
        ref={containerRef}
        style={effectStyle}
        className={cn(
          "pointer-events-none absolute inset-0 rounded-[inherit] opacity-100 transition-opacity",
          glow && "opacity-100",
          blur > 0 && "blur-[var(--blur)]",
          className,
          disabled && "!hidden",
        )}
      >
        <div
          className={cn(
            "rounded-[inherit]",
            'after:absolute after:inset-[calc(-1*var(--glowingeffect-border-width))] after:rounded-[inherit] after:content-[""]',
            "after:border-[length:var(--glowingeffect-border-width)] after:border-transparent",
            "after:bg-[image:var(--gradient)] after:[background-attachment:fixed]",
            "after:opacity-[var(--active)] after:transition-opacity after:duration-300",
            "after:[mask-composite:intersect] after:[mask-clip:padding-box,border-box]",
            "after:[mask-image:linear-gradient(#0000,#0000),conic-gradient(from_calc((var(--start)-var(--spread))*1deg),#00000000_0deg,#fff,#00000000_calc(var(--spread)*2deg))]",
          )}
        />
      </div>
    </>
  );
});
