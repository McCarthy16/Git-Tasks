import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { markdownToHtml, htmlToMarkdown } from "../markdown";
import "./MarkdownEditor.css";

interface Props {
  content?: string;
  placeholder?: string;
  onChange?: (markdown: string) => void;
  autofocus?: boolean;
}

export function MarkdownEditor({
  content = "",
  placeholder = "Write something…",
  onChange,
  autofocus = false,
}: Props) {
  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({ placeholder }),
    ],
    content: markdownToHtml(content),
    autofocus,
    onUpdate({ editor }) {
      onChange?.(htmlToMarkdown(editor.getHTML()));
    },
  });

  return (
    <div className="md-editor">
      {editor && (
        <Toolbar editor={editor} />
      )}
      <EditorContent editor={editor} className="md-editor__body" />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

import type { Editor } from "@tiptap/react";

function Toolbar({ editor }: { editor: Editor }) {
  return (
    <div className="md-editor__toolbar">
      <ToolBtn
        label="B"
        title="Bold"
        active={editor.isActive("bold")}
        onClick={() => editor.chain().focus().toggleBold().run()}
      />
      <ToolBtn
        label="I"
        title="Italic"
        active={editor.isActive("italic")}
        onClick={() => editor.chain().focus().toggleItalic().run()}
      />
      <ToolBtn
        label="S"
        title="Strike"
        active={editor.isActive("strike")}
        onClick={() => editor.chain().focus().toggleStrike().run()}
      />
      <Divider />
      <ToolBtn
        label="H1"
        title="Heading 1"
        active={editor.isActive("heading", { level: 1 })}
        onClick={() => editor.chain().focus().toggleHeading({ level: 1 }).run()}
      />
      <ToolBtn
        label="H2"
        title="Heading 2"
        active={editor.isActive("heading", { level: 2 })}
        onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()}
      />
      <ToolBtn
        label="H3"
        title="Heading 3"
        active={editor.isActive("heading", { level: 3 })}
        onClick={() => editor.chain().focus().toggleHeading({ level: 3 }).run()}
      />
      <Divider />
      <ToolBtn
        label="UL"
        title="Bullet list"
        active={editor.isActive("bulletList")}
        onClick={() => editor.chain().focus().toggleBulletList().run()}
      />
      <ToolBtn
        label="OL"
        title="Ordered list"
        active={editor.isActive("orderedList")}
        onClick={() => editor.chain().focus().toggleOrderedList().run()}
      />
      <Divider />
      <ToolBtn
        label="&lt;/&gt;"
        title="Code block"
        active={editor.isActive("codeBlock")}
        onClick={() => editor.chain().focus().toggleCodeBlock().run()}
      />
      <ToolBtn
        label="`"
        title="Inline code"
        active={editor.isActive("code")}
        onClick={() => editor.chain().focus().toggleCode().run()}
      />
      <Divider />
      <ToolBtn
        label="↩"
        title="Undo"
        active={false}
        onClick={() => editor.chain().focus().undo().run()}
      />
      <ToolBtn
        label="↪"
        title="Redo"
        active={false}
        onClick={() => editor.chain().focus().redo().run()}
      />
    </div>
  );
}

function ToolBtn({
  label,
  title,
  active,
  onClick,
}: {
  label: string;
  title: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      title={title}
      className={`md-editor__btn${active ? " is-active" : ""}`}
      onMouseDown={(e) => {
        e.preventDefault(); // keep editor focus
        onClick();
      }}
      dangerouslySetInnerHTML={{ __html: label }}
    />
  );
}

function Divider() {
  return <span className="md-editor__divider" />;
}

