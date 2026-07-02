/** Convert a subset of Markdown to TipTap-compatible HTML. */
export function markdownToHtml(md: string): string {
  if (!md.trim()) return "";

  let html = md
    .replace(/```[\s\S]*?```/g, (m) => {
      const inner = m.replace(/^```[^\n]*\n?/, "").replace(/\n?```$/, "");
      return `<pre><code>${escapeHtml(inner)}</code></pre>`;
    })
    .replace(/^### (.+)$/gm, "<h3>$1</h3>")
    .replace(/^## (.+)$/gm, "<h2>$1</h2>")
    .replace(/^# (.+)$/gm, "<h1>$1</h1>")
    .replace(/\*\*\*(.+?)\*\*\*/g, "<strong><em>$1</em></strong>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/~~(.+?)~~/g, "<s>$1</s>")
    .replace(/`(.+?)`/g, "<code>$1</code>")
    .replace(/^[*\-] (.+)$/gm, "<li>$1</li>")
    .replace(/^\d+\. (.+)$/gm, "<li>$1</li>")
    .replace(/(<li>[\s\S]*?<\/li>)(\n<li>[\s\S]*?<\/li>)*/g, (m) => `<ul>${m}</ul>`)
    .split(/\n{2,}/)
    .map((block) =>
      block.startsWith("<") ? block : `<p>${block.replace(/\n/g, "<br>")}</p>`
    )
    .join("\n");

  return html;
}

/** Convert TipTap HTML back to Markdown. */
export function htmlToMarkdown(html: string): string {
  return html
    .replace(/<h1>(.*?)<\/h1>/gi, "# $1\n")
    .replace(/<h2>(.*?)<\/h2>/gi, "## $1\n")
    .replace(/<h3>(.*?)<\/h3>/gi, "### $1\n")
    .replace(/<strong><em>(.*?)<\/em><\/strong>/gi, "***$1***")
    .replace(/<strong>(.*?)<\/strong>/gi, "**$1**")
    .replace(/<em>(.*?)<\/em>/gi, "*$1*")
    .replace(/<s>(.*?)<\/s>/gi, "~~$1~~")
    .replace(/<code>(.*?)<\/code>/gi, "`$1`")
    .replace(/<pre><code>([\s\S]*?)<\/code><\/pre>/gi, "```\n$1\n```\n")
    .replace(/<ul>([\s\S]*?)<\/ul>/gi, (_, inner) =>
      inner.replace(/<li>(.*?)<\/li>/gi, "- $1\n")
    )
    .replace(/<ol>([\s\S]*?)<\/ol>/gi, (_, inner) => {
      let i = 1;
      return inner.replace(/<li>(.*?)<\/li>/gi, () => `${i++}. $1\n`);
    })
    .replace(/<p>(.*?)<\/p>/gi, "$1\n\n")
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&nbsp;/g, " ")
    .trim();
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
