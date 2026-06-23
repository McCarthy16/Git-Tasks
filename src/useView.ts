import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Action, View } from "./types";

/**
 * Connects the UI to the server-driven backend. Loads the initial `View`, and
 * exposes a `dispatch` that sends an `Action` and applies the `View` the
 * backend returns. The backend owns all routing and state; this hook is the
 * only place the frontend talks to it.
 */
export function useView() {
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<View>("view")
      .then(setView)
      .catch((e) => setError(String(e)));
  }, []);

  const dispatch = useCallback(async (action: Action) => {
    setError(null);
    try {
      setView(await invoke<View>("dispatch", { action }));
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return { view, error, dispatch };
}
