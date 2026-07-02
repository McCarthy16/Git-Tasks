import { useState } from "react";
import type { Dispatch } from "../types";
import { basename } from "../path";
import "./SelectRepo.css";

export function SelectRepo({
  dispatch,
  recentWorkspaces,
}: {
  dispatch: Dispatch;
  recentWorkspaces: string[];
}) {
  const [busy, setBusy] = useState(false);

  async function pick() {
    setBusy(true);
    try {
      await dispatch({ type: "pick_workspace" });
    } finally {
      setBusy(false);
    }
  }

  async function openRecent(path: string) {
    await dispatch({ type: "open_workspace", path });
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
        {recentWorkspaces.length > 0 && (
          <>
            <div className="select-repo__divider" />
            <div className="select-repo__recents">
              {recentWorkspaces.map((path) => (
                <button
                  key={path}
                  className="select-repo__recent"
                  onClick={() => openRecent(path)}
                >
                  <span className="select-repo__recent-name">
                    {basename(path)}
                  </span>
                  <span className="select-repo__recent-path">{path}</span>
                </button>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
