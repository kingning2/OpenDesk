/**
 * Customer CRM page — list, detail, create and edit.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import type { ReactNode } from "react";
import {
  Button,
  Input,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  WorkspaceSplit,
  WorkspaceSplitCell,
  WorkspaceSplitGrid,
  WorkspaceSplitPane,
  WorkspaceSplitRow,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
} from "@desk/ui";
import { useT } from "../../i18n";
import {
  COOPERATION_OPTIONS,
  LIFECYCLE_OPTIONS,
  OUTREACH_OPTIONS,
  SOURCE_CHANNEL_OPTIONS,
  type CustomerFormValues,
} from "./customer-form";
import { useCustomers } from "./use-customers";

/**
 * Customer list + detail workspace.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function CustomerPage() {
  const t = useT();
  const {
    items,
    total,
    search,
    setSearch,
    loading,
    saving,
    error,
    selectedId,
    selectedProfile,
    mode,
    formValues,
    setFormValues,
    refreshList,
    selectCustomer,
    startCreate,
    startEdit,
    cancelEdit,
    saveCustomer,
  } = useCustomers();

  const isEditing = mode === "create" || mode === "edit";

  return (
    <PageScaffold fill containerPadding="none" className="space-y-0">
      <WorkspaceSplit className="min-h-0 flex-1 border-0">
        <WorkspaceSplitPane
          side="start"
          header={
            <div className="space-y-3 px-4 py-3">
              <WorkspaceSplitToolbar className="justify-between px-0 py-0">
                <WorkspaceSplitTitle>{t("customer.listTitle")}</WorkspaceSplitTitle>
                <Button size="sm" onClick={startCreate}>
                  {t("customer.create")}
                </Button>
              </WorkspaceSplitToolbar>
              <div className="flex gap-2">
                <Input
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder={t("customer.searchPlaceholder")}
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void refreshList(search)}
                  disabled={loading}
                >
                  {t("customer.search")}
                </Button>
              </div>
              <p className="text-[length:var(--text-xs)] text-muted-foreground">
                {t("customer.total", { count: total })}
              </p>
            </div>
          }
        >
          <div>
            {items.map((item) => (
              <WorkspaceSplitRow
                key={item.id}
                active={selectedId === item.id}
                onClick={() => void selectCustomer(item.id)}
              >
                <p className="truncate text-[length:var(--text-sm)] font-medium">
                  {item.display_name || item.email}
                </p>
                <p className="truncate text-[length:var(--text-xs)] text-muted-foreground">
                  {item.email}
                </p>
              </WorkspaceSplitRow>
            ))}
            {!loading && items.length === 0 ? (
              <p className="py-12 text-center text-[length:var(--text-sm)] text-muted-foreground">
                {t("customer.emptyList")}
              </p>
            ) : null}
          </div>
        </WorkspaceSplitPane>

        <WorkspaceSplitPane
          header={
            <WorkspaceSplitToolbar className="justify-between">
              <WorkspaceSplitTitle>
                {mode === "create"
                  ? t("customer.createTitle")
                  : selectedProfile?.display_name || selectedProfile?.email || t("customer.detailTitle")}
              </WorkspaceSplitTitle>
              <div className="flex gap-2">
                {mode === "view" && selectedProfile ? (
                  <Button variant="outline" size="sm" onClick={startEdit}>
                    {t("customer.edit")}
                  </Button>
                ) : null}
                {isEditing ? (
                  <>
                    <Button variant="outline" size="sm" onClick={cancelEdit} disabled={saving}>
                      {t("customer.cancel")}
                    </Button>
                    <Button size="sm" onClick={() => void saveCustomer()} disabled={saving}>
                      {saving ? t("customer.saving") : t("customer.save")}
                    </Button>
                  </>
                ) : null}
              </div>
            </WorkspaceSplitToolbar>
          }
        >
          {error ? (
            <p className="border-b border-border/50 px-4 py-3 text-[length:var(--text-sm)] text-destructive">
              {error.startsWith("customer.") ? t(error) : error}
            </p>
          ) : null}

          {mode === "empty" ? (
            <p className="py-16 text-center text-[length:var(--text-sm)] text-muted-foreground">
              {t("customer.selectHint")}
            </p>
          ) : (
            <CustomerFormFields
              values={formValues}
              readOnly={!isEditing}
              onChange={setFormValues}
              t={t}
            />
          )}
        </WorkspaceSplitPane>
      </WorkspaceSplit>
    </PageScaffold>
  );
}

interface CustomerFormFieldsProps {
  values: CustomerFormValues;
  readOnly: boolean;
  onChange: (values: CustomerFormValues) => void;
  t: (key: string) => string;
}

function CustomerFormFields({ values, readOnly, onChange, t }: CustomerFormFieldsProps) {
  const update = (patch: Partial<CustomerFormValues>) => {
    onChange({ ...values, ...patch });
  };

  return (
    <WorkspaceSplitGrid>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.displayName")}>
          <Input
            value={values.displayName}
            readOnly={readOnly}
            onChange={(event) => update({ displayName: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.email")}>
          <Input
            value={values.email}
            readOnly={readOnly}
            onChange={(event) => update({ email: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.whatsappPhone")}>
          <Input
            value={values.whatsappPhone}
            readOnly={readOnly}
            onChange={(event) => update({ whatsappPhone: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <EnumField
          label={t("customer.fields.sourceChannel")}
          value={values.sourceChannel}
          options={SOURCE_CHANNEL_OPTIONS}
          readOnly={readOnly}
          onChange={(value) => update({ sourceChannel: value })}
          t={t}
          prefix="customer.source"
        />
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <EnumField
          label={t("customer.fields.lifecycleStatus")}
          value={values.lifecycleStatus}
          options={LIFECYCLE_OPTIONS}
          readOnly={readOnly}
          onChange={(value) => update({ lifecycleStatus: value })}
          t={t}
          prefix="customer.lifecycle"
        />
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <EnumField
          label={t("customer.fields.outreachStage")}
          value={values.outreachStage}
          options={OUTREACH_OPTIONS}
          readOnly={readOnly}
          onChange={(value) => update({ outreachStage: value })}
          t={t}
          prefix="customer.outreach"
        />
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.quotedPrice")}>
          <Input
            value={values.quotedPrice}
            readOnly={readOnly}
            onChange={(event) => update({ quotedPrice: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.quotedCurrency")}>
          <Input
            value={values.quotedCurrency}
            readOnly={readOnly}
            onChange={(event) => update({ quotedCurrency: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.pricingTier")}>
          <Input
            value={values.pricingTier}
            readOnly={readOnly}
            onChange={(event) => update({ pricingTier: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <EnumField
          label={t("customer.fields.cooperationStatus")}
          value={values.cooperationStatus}
          options={COOPERATION_OPTIONS}
          readOnly={readOnly}
          onChange={(value) => update({ cooperationStatus: value })}
          t={t}
          prefix="customer.cooperation"
        />
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.packageName")}>
          <Input
            value={values.packageName}
            readOnly={readOnly}
            onChange={(event) => update({ packageName: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.monthlyFee")}>
          <Input
            value={values.monthlyFee}
            readOnly={readOnly}
            onChange={(event) => update({ monthlyFee: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.contractStart")}>
          <Input
            type="date"
            value={values.contractStart}
            readOnly={readOnly}
            onChange={(event) => update({ contractStart: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell>
        <Field label={t("customer.fields.contractEnd")}>
          <Input
            type="date"
            value={values.contractEnd}
            readOnly={readOnly}
            onChange={(event) => update({ contractEnd: event.target.value })}
          />
        </Field>
      </WorkspaceSplitCell>
      <WorkspaceSplitCell span="full">
        <Field label={t("customer.fields.notes")}>
          <textarea
            value={values.notes}
            readOnly={readOnly}
            onChange={(event) => update({ notes: event.target.value })}
            className="min-h-24 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-[length:var(--text-sm)]"
          />
        </Field>
      </WorkspaceSplitCell>
    </WorkspaceSplitGrid>
  );
}

function Field({
  label,
  children,
  className,
}: {
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <label className={cn("space-y-1.5", className)}>
      <span className="text-[length:var(--text-xs)] text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}

function EnumField({
  label,
  value,
  options,
  readOnly,
  onChange,
  t,
  prefix,
}: {
  label: string;
  value: string;
  options: readonly string[];
  readOnly: boolean;
  onChange: (value: string) => void;
  t: (key: string) => string;
  prefix: string;
}) {
  if (readOnly) {
    return (
      <Field label={label}>
        <Input value={t(`${prefix}.${value}`)} readOnly />
      </Field>
    );
  }

  return (
    <Field label={label}>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option} value={option}>
              {t(`${prefix}.${option}`)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </Field>
  );
}
