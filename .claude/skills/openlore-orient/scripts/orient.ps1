# Wrapper: ask `openlore orient` for the relevant functions/callers/specs/
# insertion-points for a task, and print the JSON to stdout.
#
# Strategy (in order):
#   1. `npx --yes openlore orient --json --task "<task>"` — preferred path,
#      uses the orient CLI subcommand.
#   2. Drive the existing `openlore mcp` server over stdio JSON-RPC via the
#      sibling orient-via-mcp.mjs helper — fallback for older openlore versions
#      that predate the CLI subcommand.

param(
  [Parameter(Mandatory=$false, Position=0)]
  [string]$Task
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($Task)) {
  [Console]::Error.WriteLine('usage: orient.ps1 "<task description>"')
  exit 2
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Preferred path: direct CLI subcommand (if/when it ships). Detect by
# scanning `openlore --help` — commander returns exit 0 even for unknown
# subcommands, so we can't rely on `orient --help`'s exit code.
$help = & npx --yes openlore --help 2>$null
if ($help -match '(?m)^  orient( |$|\[)') {
  & npx --yes openlore orient --json --task $Task
  exit $LASTEXITCODE
}

# Fallback path: drive the MCP server over stdio JSON-RPC.
& node (Join-Path $ScriptDir 'orient-via-mcp.mjs') $Task
exit $LASTEXITCODE
