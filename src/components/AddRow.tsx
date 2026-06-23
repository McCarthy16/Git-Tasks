import { useState } from "react";
import "./list-screen.css";

/**
 * Inline "+ input" row. Owns its own draft text and submits a trimmed,
 * non-empty value on Enter, then clears.
 */
export function AddRow({
  placeholder,
  onSubmit,
}: {
  placeholder: string;
  onSubmit: (name: string) => void | Promise<void>;
}) {
  const [name, setName] = useState("");

  async function submit() {
    const trimmed = name.trim();
    if (!trimmed) return;
    await onSubmit(trimmed);
    setName("");
  }

  return (
    <div className="screen__add">
      <span className="screen__add-glyph">+</span>
      <input
        className="screen__add-input"
        placeholder={placeholder}
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && submit()}
      />
    </div>
  );
}
