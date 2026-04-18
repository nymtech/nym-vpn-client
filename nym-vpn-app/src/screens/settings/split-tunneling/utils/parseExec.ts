/**
 * Parses a desktop entry Exec= value into an argument list suitable for
 * passing to Command.create().
 *
 * Per the XDG Desktop Entry Specification:
 * - If the executable path contains spaces it MUST be wrapped in double quotes
 *   and spaces inside the quoted path MUST be escaped as \s.
 * - Arguments following the executable are separated by regular spaces.
 *
 * Examples:
 *   '/usr/bin/firefox'               → ['/usr/bin/firefox']
 *   '/usr/bin/app --flag'            → ['/usr/bin/app', '--flag']
 *   '"/opt/My\sApp/app" --flag'      → ['/opt/My App/app', '--flag']
 */
export function parseExecArgs(exec: string): string[] {
  const trimmed = exec.trim();

  if (trimmed.startsWith('"')) {
    const closingQuote = trimmed.indexOf('"', 1);
    if (closingQuote !== -1) {
      const path = trimmed.slice(1, closingQuote).replace(/\\s/g, ' ');
      const rest = trimmed.slice(closingQuote + 1).trim();
      return [path, ...rest.split(' ').filter(Boolean)];
    }
  }

  // Unquoted path: split on spaces as before
  return trimmed.split(' ').filter(Boolean);
}
