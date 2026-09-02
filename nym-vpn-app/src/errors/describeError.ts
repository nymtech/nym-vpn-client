// Renders an unknown thrown value as text safe to show and to log.
// Anything can be thrown in JS, so this must never throw itself: `instanceof`
// runs the value's `getPrototypeOf` trap and `Object.prototype.toString` reads
// its `Symbol.toStringTag`, either of which throws for a hostile or revoked
// proxy. Every inspection therefore sits behind a guard.
export function describeError(error: unknown): string | null {
  if (error === null || error === undefined) {
    return null;
  }
  try {
    if (error instanceof Error) {
      return error.stack || `${error.name}: ${error.message}`;
    }
    if (typeof error === 'string') {
      return error;
    }
    // JSON.stringify returns undefined for functions and symbols
    return (
      JSON.stringify(error, null, 2) ?? Object.prototype.toString.call(error)
    );
  } catch {
    // circular structures, BigInt, exotic proxies
    try {
      return Object.prototype.toString.call(error);
    } catch {
      // revoked proxy: nothing about it can be read at all
      return '<unprintable error>';
    }
  }
}
