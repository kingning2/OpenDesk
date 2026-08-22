/**
 * License feature 公开导出。
 *
 * @author coisini
 * @created 2026-07-16
 */

export { LicenseGateProvider, useLicenseGateContext } from "./license-gate-context";
export { LicenseActivationPanel } from "./license-activation-panel";
export { LicensePlanBadge } from "./license-plan-badge";
export { LicensePlanDialog } from "./license-plan-dialog";
export type { LicensePlanDialogProps } from "./license-plan-dialog";
export {
  formatLicenseRemaining,
  formatLicenseRemainingShort,
  formatLicenseRemainingDetailed,
  formatLicenseExpiresAt,
} from "./format-license-remaining";
export type { LicenseRemainingLabel } from "./format-license-remaining";
export { useLicenseGate } from "./use-license-gate";
export type { UseLicenseGateResult } from "./use-license-gate";
export { useLicenseActivate } from "./use-license-activate";
export type { UseLicenseActivateResult } from "./use-license-activate";
export { LicenseGateController } from "./license-gate-controller";
export { LicenseActivationService } from "./license-activation-service";
