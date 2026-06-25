import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

export function RenameDialog({
  name,
  onConfirm,
  onCancel,
}: {
  name: string;
  onConfirm: (newName: string) => void;
  onCancel: () => void;
}) {
  const [value, setValue] = useState(name);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.select();
  }, []);

  const handleSubmit = () => {
    const trimmed = value.trim();
    if (trimmed && trimmed !== name) onConfirm(trimmed);
    else onCancel();
  };

  return createPortal(
    <div className="dialog-backdrop" onPointerDown={onCancel}>
      <div className="popup dialog" onPointerDown={(e) => e.stopPropagation()}>
        <div className="popup__header">Rename</div>
        <div className="dialog__body">
          <input
            ref={inputRef}
            className="dialog__input"
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
            disabled={!value.trim() || value.trim() === name}
          >
            Rename
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
