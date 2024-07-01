export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Capitalize the first letter of a string
export function capFirst(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}
