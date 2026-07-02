import type { ReactNode } from "react";

/** A vertical list of [`Row`](./Row) items. */
export function List({ children }: { children: ReactNode }) {
  return (
    <div className="list" role="list">
      {children}
    </div>
  );
}
