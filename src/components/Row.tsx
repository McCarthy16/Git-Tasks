/**
 * A single list row: a status glyph, a name, and a muted id. Renders as a
 * button when `onClick` is provided (e.g. a project you can open), otherwise as
 * a static row (e.g. a task).
 */
export function Row({
  name,
  secondary,
  chip,
  onClick,
  onContextMenu,
}: {
  name: string;
  id: string;
  secondary?: string;
  chip?: string;
  onClick?: () => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}) {
  const content = (
    <>
      <span className="row__glyph" aria-hidden="true" />
      <span className="row__name">{name}</span>
      {secondary && <span className="row__id">{secondary}</span>}
      {chip && <span className="row__chip">{chip}</span>}
    </>
  );

  return onClick ? (
    <button className="row" role="listitem" onClick={onClick} onContextMenu={onContextMenu}>
      {content}
    </button>
  ) : (
    <div className="row" role="listitem" onContextMenu={onContextMenu}>
      {content}
    </div>
  );
}
