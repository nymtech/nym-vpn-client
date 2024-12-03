export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Capitalize the first letter of a string
export function capFirst(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

// Given a set of strings, return the strings concatenated by a white space
export function setToString(obj: Record<string, string>): string {
  return Object.values(obj).reduce((prev, s) => `${prev} ${s}`, '');
}
