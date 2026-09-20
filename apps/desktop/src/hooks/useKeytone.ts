import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import type { AppSnapshot, Effects } from "../types";

export function useKeytone() {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.getState().then(setSnapshot).catch((reason: unknown) => setError(String(reason)));
  }, []);

  const act = useCallback(async (operation: () => Promise<AppSnapshot>) => {
    setBusy(true);
    setError(null);
    try {
      const next = await operation();
      setSnapshot(next);
      return next;
    } catch (reason) {
      setError(String(reason));
      throw reason;
    } finally {
      setBusy(false);
    }
  }, []);

  const updateEffects = useCallback((effects: Effects) => act(() => api.updateEffects(effects)), [act]);

  return { snapshot, setSnapshot, error, setError, busy, act, updateEffects };
}
