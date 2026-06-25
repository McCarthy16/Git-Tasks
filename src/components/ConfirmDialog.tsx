import { createPortal } from "react-dom";

export function ConfirmDialog({
  name,
  message,
  confirmLabel = "Remove",
  onConfirm,
  onCancel,
}: {
  name: string;
  message: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return createPortal(
    <div className="dialog-backdrop" onPointerDown={onCancel}>
      <div className="popup dialog" onPointerDown={(e) => e.stopPropagation()}>
        <div className="popup__header">{name}</div>
        <div className="dialog__body">{message}</div>
        <div className="dialog__actions">
          <button className="dialog__btn dialog__btn--cancel" onClick={onCancel}>
            Cancel
          </button>
          <button className="dialog__btn dialog__btn--confirm" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
