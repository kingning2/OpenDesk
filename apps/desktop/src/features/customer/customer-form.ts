/**
 * Customer profile form field values.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import type { CustomerProfile } from "@desk/platform/ipc/customer";

export interface CustomerFormValues {
  displayName: string;
  email: string;
  whatsappPhone: string;
  sourceChannel: string;
  lifecycleStatus: string;
  outreachStage: string;
  quotedPrice: string;
  quotedCurrency: string;
  quotedAt: string;
  pricingTier: string;
  cooperationStatus: string;
  packageName: string;
  monthlyFee: string;
  contractStart: string;
  contractEnd: string;
  notes: string;
}

export const LIFECYCLE_OPTIONS = [
  "new",
  "contacted",
  "negotiating",
  "won",
  "lost",
  "paused",
] as const;

export const OUTREACH_OPTIONS = [
  "no_stage",
  "stage1",
  "stage2",
  "stage3",
  "stage4",
  "stage5",
  "stage6",
  "stage7",
  "s_bounce",
  "archived",
] as const;

export const COOPERATION_OPTIONS = [
  "none",
  "negotiating",
  "active",
  "paused",
  "terminated",
] as const;

export const SOURCE_CHANNEL_OPTIONS = ["manual", "youtube"] as const;

/**
 * Empty form defaults for creating a customer.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function emptyCustomerFormValues(): CustomerFormValues {
  return {
    displayName: "",
    email: "",
    whatsappPhone: "",
    sourceChannel: "manual",
    lifecycleStatus: "new",
    outreachStage: "no_stage",
    quotedPrice: "",
    quotedCurrency: "USD",
    quotedAt: "",
    pricingTier: "",
    cooperationStatus: "none",
    packageName: "",
    monthlyFee: "",
    contractStart: "",
    contractEnd: "",
    notes: "",
  };
}

/**
 * Map API profile to editable form values.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function profileToFormValues(profile: CustomerProfile): CustomerFormValues {
  return {
    displayName: profile.display_name ?? "",
    email: profile.email,
    whatsappPhone: profile.whatsapp_phone ?? "",
    sourceChannel: profile.source_channel,
    lifecycleStatus: profile.lifecycle_status,
    outreachStage: profile.outreach_stage,
    quotedPrice: profile.quoted_price != null ? String(profile.quoted_price) : "",
    quotedCurrency: profile.quoted_currency ?? "",
    quotedAt: profile.quoted_at ?? "",
    pricingTier: profile.pricing_tier ?? "",
    cooperationStatus: profile.cooperation_status,
    packageName: profile.package_name ?? "",
    monthlyFee: profile.monthly_fee != null ? String(profile.monthly_fee) : "",
    contractStart: profile.contract_start ?? "",
    contractEnd: profile.contract_end ?? "",
    notes: profile.notes ?? "",
  };
}

/**
 * Map form values to create IPC payload.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function formValuesToCreatePayload(values: CustomerFormValues) {
  return {
    display_name: values.displayName.trim() || undefined,
    email: values.email.trim(),
    whatsapp_phone: values.whatsappPhone.trim() || undefined,
    source_channel: values.sourceChannel,
    lifecycle_status: values.lifecycleStatus,
    outreach_stage: values.outreachStage,
    quoted_price: parseOptionalNumber(values.quotedPrice),
    quoted_currency: values.quotedCurrency.trim() || undefined,
    quoted_at: values.quotedAt.trim() || undefined,
    pricing_tier: values.pricingTier.trim() || undefined,
    cooperation_status: values.cooperationStatus,
    package_name: values.packageName.trim() || undefined,
    monthly_fee: parseOptionalNumber(values.monthlyFee),
    contract_start: values.contractStart.trim() || undefined,
    contract_end: values.contractEnd.trim() || undefined,
    notes: values.notes.trim() || undefined,
  };
}

/**
 * Map form values to update IPC payload.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function formValuesToUpdatePayload(id: string, values: CustomerFormValues) {
  return {
    id,
    ...formValuesToCreatePayload(values),
  };
}

function parseOptionalNumber(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : undefined;
}
