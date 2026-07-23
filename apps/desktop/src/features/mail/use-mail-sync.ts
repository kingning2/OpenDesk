/**
 * Poll IMAP sync status and trigger manual sync.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { useCallback, useEffect, useRef, useState } from "react";
import {
  mailSyncNow,
  mailSyncStatus,
  type MailImapSyncState,
} from "@desk/platform";

const POLL_MS = 30_000;

/**
 * Track IMAP sync state for the mail workbench.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param accountId - Optional account filter
 * @returns Sync status rows, loading flags, and actions
 */
export function useMailSync(accountId?: string | null) {
  const [items, setItems] = useState<MailImapSyncState[]>([]);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const response = await mailSyncStatus({
        account_id: accountId || undefined,
      });
      if (mountedRef.current) {
        setItems(response.items);
      }
    } finally {
      if (mountedRef.current) {
        setLoading(false);
      }
    }
  }, [accountId]);

  const syncNow = useCallback(async () => {
    setSyncing(true);
    try {
      await mailSyncNow({ account_id: accountId || undefined });
      await refresh();
    } finally {
      if (mountedRef.current) {
        setSyncing(false);
      }
    }
  }, [accountId, refresh]);

  useEffect(() => {
    mountedRef.current = true;
    let cancelled = false;
    void mailSyncStatus({
      account_id: accountId || undefined,
    })
      .then((response) => {
        if (!cancelled && mountedRef.current) {
          setItems(response.items);
        }
      })
      .catch(() => {
        /* keep previous items on poll failure */
      });
    const timer = window.setInterval(() => {
      void mailSyncStatus({
        account_id: accountId || undefined,
      })
        .then((response) => {
          if (mountedRef.current) {
            setItems(response.items);
          }
        })
        .catch(() => {
          /* keep previous items on poll failure */
        });
    }, POLL_MS);
    return () => {
      cancelled = true;
      mountedRef.current = false;
      window.clearInterval(timer);
    };
  }, [accountId]);

  const isSyncing = items.some((item) => item.is_syncing) || syncing;
  const lastError = items.find((item) => item.last_error)?.last_error;
  const lastSyncAt = items
    .map((item) => item.last_sync_at)
    .filter((value): value is string => Boolean(value))
    .sort()
    .slice(-1)[0];

  return {
    items,
    loading,
    isSyncing,
    lastError,
    lastSyncAt,
    refresh,
    syncNow,
  };
}
