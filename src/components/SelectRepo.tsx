import { useState } from "react";
import type { Dispatch } from "../types";
import "./SelectRepo.css";

/**
 * Screen 1: shown when no workspace is open. The button dispatches
 * `pick_workspace`; the backend opens the native folder picker, finds-or-creates
 * `.tasks`, and returns the next view.
 */
export function SelectRepo({ dispatch }: { dispatch: Dispatch }) {
  const [busy, setBusy] = useState(false);

  async function pick() {
    setBusy(true);
    try {
      await dispatch({ type: "pick_workspace" });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="select-repo">
      <div className="select-repo__card">
        <h1 className="select-repo__title">tasks</h1>
        <p className="select-repo__subtitle">
          Choose the repository you want to work in. A <code>.tasks</code>{" "}
          folder will be created there if one doesn't exist yet.
        </p>
        <button
          className="select-repo__button"
          onClick={pick}
          disabled={busy}
        >
          {busy ? "Opening…" : "Select Repo"}
        </button>
      </div>
    </div>
  );
}
