import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

export function CreateProjectDialog({
  onConfirm,
  onCancel,
}: {
  onConfirm: (name: string) => void;
  onCancel: () => void;
}) {
  const [value, setValue] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = () => {
    const trimmed = value.trim();
    if (trimmed) onConfirm(trimmed);
    else onCancel();
  };

  return createPortal(
    <div className="dialog-backdrop" onPointerDown={onCancel}>
      <div className="popup dialog" onPointerDown={(e) => e.stopPropagation()}>
        <div className="popup__header">New project</div>
        <div className="dialog__body">
          <input
            ref={inputRef}
            className="dialog__input"
            placeholder="Project name…"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSubmit();
              if (e.key === "Escape") onCancel();
            }}
          />
        </div>
        <div className="dialog__actions">
          <button className="dialog__btn dialog__btn--cancel" onClick={onCancel}>
            Cancel
          </button>
          <button
            className="dialog__btn dialog__btn--confirm"
            onClick={handleSubmit}
            disabled={!value.trim()}
          >
            Create
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
