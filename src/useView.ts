import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Action, View } from "./types";

export function useView() {
  const [view, setView] = useState<View | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<View>("view")
      .then(setView)
      .catch((e) => setError(String(e)));
  }, []);

  // Refresh the view when a dock menu / system-level event opens a workspace
  // in this instance (macOS dock menu clicks arrive via this channel).
  useEffect(() => {
    const unlisten = listen<void>("view-updated", () => {
      invoke<View>("view")
        .then(setView)
        .catch((e) => setError(String(e)));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const dispatch = useCallback(async (action: Action): Promise<View | null> => {
    setError(null);
    try {
      const newView = await invoke<View>("dispatch", { action });
      setView(newView);
      return newView;
    } catch (e) {
      setError(String(e));
      return null;
    }
  }, []);

  return { view, error, dispatch };
}
