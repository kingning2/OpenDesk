/**
 * Customer list + detail state hook.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import { useCallback, useEffect, useState } from "react";
import {
  customerCreate,
  customerGet,
  customerList,
  customerUpdate,
  type CustomerProfile,
} from "@desk/platform/ipc/customer";
import type { CustomerFormValues } from "./customer-form";
import {
  emptyCustomerFormValues,
  formValuesToCreatePayload,
  formValuesToUpdatePayload,
  profileToFormValues,
} from "./customer-form";

export type CustomerPanelMode = "empty" | "create" | "view" | "edit";

/**
 * Customer page state and mutations.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export function useCustomers() {
  const [items, setItems] = useState<CustomerProfile[]>([]);
  const [total, setTotal] = useState(0);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedProfile, setSelectedProfile] = useState<CustomerProfile | null>(null);
  const [mode, setMode] = useState<CustomerPanelMode>("empty");
  const [formValues, setFormValues] = useState<CustomerFormValues>(emptyCustomerFormValues());

  const refreshList = useCallback(async (query = search) => {
    setLoading(true);
    setError(null);
    try {
      const result = await customerList({
        search: query.trim() || undefined,
        limit: 100,
        offset: 0,
      });
      setItems(result.items);
      setTotal(result.total);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setLoading(false);
    }
  }, [search]);

  useEffect(() => {
    let cancelled = false;
    void customerList({ limit: 100, offset: 0 })
      .then((result) => {
        if (!cancelled) {
          setItems(result.items);
          setTotal(result.total);
        }
      })
      .catch((cause) => {
        if (!cancelled) {
          setError(String(cause));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const selectCustomer = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const profile = await customerGet(id);
      setSelectedId(id);
      setSelectedProfile(profile);
      setFormValues(profileToFormValues(profile));
      setMode("view");
    } catch (cause) {
      setError(String(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  const startCreate = useCallback(() => {
    setSelectedId(null);
    setSelectedProfile(null);
    setFormValues(emptyCustomerFormValues());
    setMode("create");
    setError(null);
  }, []);

  const startEdit = useCallback(() => {
    if (selectedProfile) {
      setFormValues(profileToFormValues(selectedProfile));
      setMode("edit");
      setError(null);
    }
  }, [selectedProfile]);

  const cancelEdit = useCallback(() => {
    if (selectedProfile) {
      setFormValues(profileToFormValues(selectedProfile));
      setMode("view");
    } else {
      setMode("empty");
    }
    setError(null);
  }, [selectedProfile]);

  const saveCustomer = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      if (mode === "create") {
        const profile = await customerCreate(formValuesToCreatePayload(formValues));
        await refreshList();
        setSelectedId(profile.id);
        setSelectedProfile(profile);
        setFormValues(profileToFormValues(profile));
        setMode("view");
        return;
      }

      if ((mode === "edit" || mode === "view") && selectedId) {
        const profile = await customerUpdate(
          formValuesToUpdatePayload(selectedId, formValues),
        );
        await refreshList();
        setSelectedProfile(profile);
        setFormValues(profileToFormValues(profile));
        setMode("view");
      }
    } catch (cause) {
      const message = String(cause);
      if (message.includes("customer.email_duplicate")) {
        setError("customer.emailDuplicate");
      } else if (message.includes("customer.email_required")) {
        setError("customer.emailRequired");
      } else {
        setError(message);
      }
    } finally {
      setSaving(false);
    }
  }, [formValues, mode, refreshList, selectedId]);

  return {
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
  };
}
