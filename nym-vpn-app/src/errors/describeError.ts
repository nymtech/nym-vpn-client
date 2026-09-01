// Renders an unknown thrown value as text safe to show and to log.
// Anything can be thrown in JS, so this must never throw itself.
export function describeError(error: unknown): string | null {
  if (error === null || error === undefined) {
    return null;
  }
  if (error instanceof Error) {
    return error.stack || `${error.name}: ${error.message}`;
  }
  if (typeof error === 'string') {
    return error;
  }
  try {
    // JSON.stringify returns undefined for functions and symbols
    return (
      JSON.stringify(error, null, 2) ?? Object.prototype.toString.call(error)
    );
  } catch {
    // circular structures, BigInt, exotic proxies
    return Object.prototype.toString.call(error);
  }
}
