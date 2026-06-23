/**
 * A single list row: a status glyph, a name, and a muted id. Renders as a
 * button when `onClick` is provided (e.g. a project you can open), otherwise as
 * a static row (e.g. a task).
 */
export function Row({
  name,
  id,
  onClick,
}: {
  name: string;
  id: string;
  onClick?: () => void;
}) {
  const content = (
    <>
      <span className="row__glyph" aria-hidden="true" />
      <span className="row__name">{name}</span>
      <span className="row__id">{id}</span>
    </>
  );

  return onClick ? (
    <button className="row" role="listitem" onClick={onClick}>
      {content}
    </button>
  ) : (
    <div className="row" role="listitem">
      {content}
    </div>
  );
}
