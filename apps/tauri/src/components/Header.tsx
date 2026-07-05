import "./header.css";

export type Crumb = {
  label: string;
  /** Omit for the current location (rendered as plain text, not a link). */
  onClick?: () => void;
};

/** Slim header above the main pane: a left-aligned breadcrumb trail. */
export function Header({ crumbs }: { crumbs: Crumb[] }) {
  return (
    <header className="header">
      <nav className="crumbs">
        {crumbs.map((crumb, i) => (
          <span className="crumbs__item" key={i}>
            {i > 0 && (
              <span className="crumbs__sep" aria-hidden="true">
                /
              </span>
            )}
            {crumb.onClick ? (
              <button className="crumbs__link" onClick={crumb.onClick}>
                {crumb.label}
              </button>
            ) : (
              <span className="crumbs__current">{crumb.label}</span>
            )}
          </span>
        ))}
      </nav>
    </header>
  );
}
