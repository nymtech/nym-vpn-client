import { describeError } from './describeError';

// Errors that no React boundary can catch: unhandled promise rejections and
// throws from event handlers, timers or Tauri event callbacks. They are only
// logged. console.* is forwarded to the Rust logger, so they land in the logs
// archive the error screen's export button produces. Deciding to replace a
// working UI over a rejection we cannot attribute is deliberately not done:
// the fatal screen is reserved for a failed startup and the error boundary.

// Browsers report this as a window `error` event although it is a layout
// warning, not a failure. It is routinely triggered by observers measuring
// elements they also resize.
const ResizeObserverLoop = 'ResizeObserver loop';

let installed = false;

export function installGlobalErrorHandlers(): void {
  if (installed) {
    return;
  }
  installed = true;

  window.addEventListener('unhandledrejection', (event) => {
    console.error(`unhandled rejection: ${describeError(event.reason)}`);
  });

  window.addEventListener('error', (event) => {
    if (event.message?.includes(ResizeObserverLoop)) {
      return;
    }
    console.error(
      `unhandled error: ${describeError(event.error ?? event.message)}`,
    );
  });
}
