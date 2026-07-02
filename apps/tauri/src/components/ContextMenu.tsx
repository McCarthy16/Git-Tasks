import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";

export function ContextMenu({
  name,
  id,
  x,
  y,
  onClose,
  onRename,
  onMove,
  onChangeStatus,
  onRemove,
}: {
  name: string;
  id: string;
  x: number;
  y: number;
  onClose: () => void;
  onRename: () => void;
  onMove?: () => void;
  onChangeStatus?: () => void;
  onRemove: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handlePointerDown = (e: PointerEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  return createPortal(
    <div ref={ref} className="popup context-menu" style={{ left: x, top: y }}>
      <div className="popup__header">{name}</div>
      <div className="popup__items">
        <button
          className="popup__item"
          onClick={() => { onClose(); onRename(); }}
        >
          Rename
        </button>
        {onMove && (
          <button
            className="popup__item"
            onClick={() => { onClose(); onMove(); }}
          >
            Move to project…
          </button>
        )}
        {onChangeStatus && (
          <button
            className="popup__item"
            onClick={() => { onClose(); onChangeStatus(); }}
          >
            Change status…
          </button>
        )}
        <button
          className="popup__item popup__item--danger"
          onClick={() => { onClose(); onRemove(); }}
        >
          Remove
        </button>
      </div>
      <div className="popup__footer">{id}</div>
    </div>,
    document.body
  );
}
