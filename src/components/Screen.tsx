import type { ReactNode } from "react";
import { Header, type Crumb } from "./Header";
import "./list-screen.css";

/**
 * The shared screen shell: a breadcrumb header above a scrollable body. An
 * optional error banner renders at the top of the body.
 */
export function Screen({
  crumbs,
  error,
  children,
}: {
  crumbs: Crumb[];
  error?: string | null;
  children: ReactNode;
}) {
  return (
    <div className="screen">
      <Header crumbs={crumbs} />
      <div className="screen__body">
        {error && <p className="screen__error">{error}</p>}
        {children}
      </div>
    </div>
  );
}
