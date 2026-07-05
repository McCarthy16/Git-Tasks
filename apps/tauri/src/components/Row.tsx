/**
 * A single list row: a status glyph, a name, and a muted id. Renders as a
 * button when `onClick` is provided (e.g. a project you can open), otherwise as
 * a static row (e.g. a task). The chip becomes clickable when `onChipClick` is
 * provided (rendered as a span with a button role, since the row itself may
 * already be a button).
 */
export function Row({
  name,
  secondary,
  chip,
  onClick,
  onChipClick,
  onContextMenu,
}: {
  name: string;
  id: string;
  secondary?: string;
  chip?: string;
  onClick?: () => void;
  onChipClick?: () => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}) {
  const content = (
    <>
      <span className="row__glyph" aria-hidden="true" />
      <span className="row__name">{name}</span>
      {secondary && <span className="row__id">{secondary}</span>}
      {chip !== undefined &&
        (onChipClick ? (
          <span
            className="row__chip row__chip--btn"
            role="button"
            tabIndex={0}
            onClick={(e) => {
              e.stopPropagation();
              onChipClick();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                e.stopPropagation();
                onChipClick();
              }
            }}
          >
            {chip}
          </span>
        ) : (
          <span className="row__chip">{chip}</span>
        ))}
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
