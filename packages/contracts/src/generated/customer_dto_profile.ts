export interface CustomerDtoProfile {
  id: string;
  display_name?: string;
  email: string;
  whatsapp_phone?: string;
  source_channel: string;
  source_meta?: string;
  lifecycle_status: string;
  outreach_stage: string;
  quoted_price?: number;
  quoted_currency?: string;
  quoted_at?: string;
  pricing_tier?: string;
  cooperation_status: string;
  package_name?: string;
  monthly_fee?: number;
  contract_start?: string;
  contract_end?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}
