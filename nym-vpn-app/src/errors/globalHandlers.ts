import { describeError } from './describeError';

// Escalation policy for errors that no React boundary can catch.
//
// A stray unhandled rejection is usually benign — an aborted request, a Tauri
// call cancelled by navigation, a listener firing after unmount. Replacing a
// working UI with an error screen over one of those would make the app look
// far more broken than it is, and could tear down a screen while a tunnel is
// connected. So the full screen is only shown when the UI is demonstrably not
// working: React never mounted, or the boundary has already tripped.

type Listener = (error: unknown) => void;

let uiMounted = false;
let treeDown = false;
let escalate: Listener | null = null;
const listeners = new Set<Listener>();

/** Called once React has successfully rendered the app. */
export function markUiMounted(): void {
  uiMounted = true;
}

/** Called by the error boundary when the tree has come down. */
export function markTreeDown(): void {
  treeDown = true;
}

/** How a caller renders the fatal error screen when we decide to escalate. */
export function setEscalationHandler(handler: Listener): void {
  escalate = handler;
}

/** Subscribed to from inside the provider tree to surface non-fatal errors. */
export function onNonFatalError(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function handle(error: unknown, source: string) {
  console.error(`unhandled ${source}: ${describeError(error)}`);

  if (!uiMounted || treeDown) {
    treeDown = true;
    escalate?.(error);
    return;
  }

  listeners.forEach((listener) => {
    try {
      listener(error);
    } catch (e) {
      // a failing listener must never re-enter this handler
      console.error(`non-fatal error listener failed: ${describeError(e)}`);
    }
  });
}

let installed = false;

export function installGlobalErrorHandlers(): void {
  if (installed) {
    return;
  }
  installed = true;

  window.addEventListener('unhandledrejection', (event) => {
    handle(event.reason, 'rejection');
  });

  window.addEventListener('error', (event) => {
    // resource load failures (img, script) also fire here but carry no error
    handle(event.error ?? event.message, 'error');
  });
}
