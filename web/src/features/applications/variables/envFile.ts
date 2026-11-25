export interface ParsedVariable {
  key: string;
  value: string;
}

const LINE = /^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)\s*$/;

function unquote(raw: string): string {
  const trimmed = raw.trim();
  const quote = trimmed[0];
  if ((quote === '"' || quote === "'") && trimmed.length >= 2 && trimmed.endsWith(quote)) {
    const inner = trimmed.slice(1, -1);
    return quote === '"' ? inner.replace(/\\n/g, "\n").replace(/\\"/g, '"').replace(/\\\\/g, "\\") : inner;
  }
  const comment = trimmed.search(/\s#/);
  return comment === -1 ? trimmed : trimmed.slice(0, comment).trimEnd();
}

export function parseEnvFile(text: string): { variables: ParsedVariable[]; invalidLines: number[] } {
  const variables = new Map<string, string>();
  const invalidLines: number[] = [];
  text.split(/\r?\n/).forEach((line, index) => {
    if (line.trim() === "" || line.trim().startsWith("#")) {
      return;
    }
    const match = LINE.exec(line);
    if (match?.[1] === undefined) {
      invalidLines.push(index + 1);
      return;
    }
    variables.set(match[1], unquote(match[2] ?? ""));
  });
  return { variables: [...variables].map(([key, value]) => ({ key, value })), invalidLines };
}

function quote(value: string): string {
  return /^[A-Za-z0-9_./:@-]*$/.test(value) ? value : `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")}"`;
}

export function serializeEnvFile(variables: ParsedVariable[]): string {
  return variables.map(({ key, value }) => `${key}=${quote(value)}`).join("\n");
}
