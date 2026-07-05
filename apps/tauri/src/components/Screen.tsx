import type { ReactNode } from "react";
import "./list-screen.css";

/**
 * The shared pane shell rendered inside the app shell's main area: a
 * scrollable body with an optional error banner at the top. Orientation
 * (breadcrumbs) lives in the shell's header, not here.
 */
export function Screen({
  error,
  children,
}: {
  error?: string | null;
  children: ReactNode;
}) {
  return (
    <div className="screen">
      <div className="screen__body">
        {error && <p className="screen__error">{error}</p>}
        {children}
      </div>
    </div>
  );
}
