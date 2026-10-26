export function markdownTable(text: string, language?: string | null): string[][] | null {
  if (language !== 'markdown' || text.length > 65536) return null;
  const lines = text.trim().split(/\r?\n/);
  if (lines.length < 2 || lines.length > 102 || lines.some(line => !/^\s*\|.*\|\s*$/.test(line))) return null;
  function cells(line: string): string[] {
    const values: string[] = []; let current = '', escaped = false;
    for (const char of line.trim().slice(1, -1)) {
      if (escaped) { current += char === '|' ? char : `\\${char}`; escaped = false; }
      else if (char === '\\') escaped = true;
      else if (char === '|') { values.push(current.trim()); current = ''; }
      else current += char;
    }
    if (escaped) current += '\\'; values.push(current.trim()); return values;
  }
  const rows = lines.map(cells), width = rows[0].length;
  if (!width || width > 20 || rows.some(row => row.length !== width) || !rows[1].every(cell => /^:?-{3,}:?$/.test(cell))) return null;
  return rows.filter((_, i) => i !== 1);
}
