import { useEffect } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { markdownToHtml } from "../markdown";
import { formatAbsoluteTime } from "../time";
import type { TaskEventEntry } from "../types";
import "./DescriptionSnapshotDialog.css";

/** Modal showing the rendered markdown content of a description_updated event. */
export function DescriptionSnapshotDialog({
  event,
  onClose,
}: {
  event: TaskEventEntry;
  onClose: () => void;
}) {
  const editor = useEditor({
    extensions: [StarterKit],
    content: markdownToHtml(event.detail ?? ""),
    editable: false,
  });

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="dialog-backdrop" onMouseDown={onClose}>
      <div className="desc-snapshot popup" onMouseDown={(e) => e.stopPropagation()}>
        <div className="popup__header desc-snapshot__header">
          <span>Description snapshot</span>
          {event.created_at_millis != null && (
            <span className="desc-snapshot__timestamp">
              {formatAbsoluteTime(event.created_at_millis)}
            </span>
          )}
        </div>
        <div className="desc-snapshot__body md-editor__body">
          {event.detail ? (
            <EditorContent editor={editor} />
          ) : (
            <p className="desc-snapshot__empty">No description recorded.</p>
          )}
        </div>
      </div>
    </div>
  );
}
