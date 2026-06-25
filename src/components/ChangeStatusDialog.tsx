import { useState } from "react";
import { createPortal } from "react-dom";
import type { Status } from "../types";

const NO_STATUS = "";

export function ChangeStatusDialog({
  name,
  currentStatusId,
  statuses,
  onConfirm,
  onCancel,
}: {
  name: string;
  currentStatusId: string | null;
  statuses: Status[];
  onConfirm: (statusId: string | null) => void;
  onCancel: () => void;
}) {
  const [selected, setSelected] = useState(currentStatusId ?? NO_STATUS);

  return createPortal(
    <div className="dialog-backdrop" onPointerDown={onCancel}>
      <div className="popup dialog" onPointerDown={(e) => e.stopPropagation()}>
        <div className="popup__header">{name}</div>
        <div className="dialog__body">
          <select
            className="dialog__select"
            value={selected}
            onChange={(e) => setSelected(e.target.value)}
          >
            <option value={NO_STATUS}>No Status</option>
            {statuses.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </div>
        <div className="dialog__actions">
          <button className="dialog__btn dialog__btn--cancel" onClick={onCancel}>
            Cancel
          </button>
          <button
            className="dialog__btn dialog__btn--confirm"
            onClick={() => onConfirm(selected === NO_STATUS ? null : selected)}
          >
            Set Status
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
