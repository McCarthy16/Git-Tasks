import { useState } from "react";
import { createPortal } from "react-dom";
import type { Project } from "../types";

export function MoveDialog({
  name,
  currentProjectId,
  projects,
  onConfirm,
  onCancel,
}: {
  name: string;
  currentProjectId: string;
  projects: Project[];
  onConfirm: (projectId: string) => void;
  onCancel: () => void;
}) {
  const options = projects.filter((p) => p.id !== currentProjectId);
  const [selected, setSelected] = useState(options[0]?.id ?? "");

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
            {options.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
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
            onClick={() => onConfirm(selected)}
            disabled={!selected}
          >
            Move
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
