/** 生成 UUID v4（浏览器 `crypto.randomUUID`）。 */
export function uuid(): string {
  return crypto.randomUUID();
}
