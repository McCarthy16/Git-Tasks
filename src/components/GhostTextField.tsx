import { useEffect, useRef, useState } from "react";
import "./GhostTextField.css";

interface GhostTextFieldProps {
  value: string;
  onCommit: (value: string) => void;
  className?: string;
  placeholder?: string;
}

export function GhostTextField({ value, onCommit, className, placeholder }: GhostTextFieldProps) {
  const [draft, setDraft] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  function commit() {
    const trimmed = draft.trim();
    if (trimmed && trimmed !== value) {
      onCommit(trimmed);
    } else {
      setDraft(value);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      e.preventDefault();
      inputRef.current?.blur();
    }
    if (e.key === "Escape") {
      setDraft(value);
      inputRef.current?.blur();
    }
  }

  return (
    <input
      ref={inputRef}
      className={`ghost-text-field ${className ?? ""}`.trim()}
      value={draft}
      placeholder={placeholder}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={commit}
      onKeyDown={handleKeyDown}
      onFocus={(e) => e.target.select()}
    />
  );
}
