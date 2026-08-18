import type { SwSection } from '@/lib/sections';

export interface ScrollWorldConfig {
  brand?: { name?: string; href?: string };
  cta?: { label?: string; href?: string };
  hint?: string;
  diveScroll?: number;
  connScroll?: number;
  crossfade?: number;
  nav?: boolean;
  atmosphere?: boolean;
  sections: SwSection[];
  connectors?: (string | null)[];
  connectorsMobile?: (string | null)[];
}

export declare function mountScrollWorld(
  container: HTMLElement,
  config: ScrollWorldConfig,
): void;
