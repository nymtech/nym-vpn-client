import React from 'react';
import { FatalError } from '../screens';
import { describeError, markTreeDown } from '../errors';

type Props = { children: React.ReactNode };
type State = { error: unknown; crashed: boolean };

// Catches render and lifecycle throws from the whole app, including the
// provider tree. Mounted above `App` in main.tsx, because a throw from any of
// the providers wrapping the router would otherwise unmount everything and
// leave a blank window.
class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null, crashed: false };

  static getDerivedStateFromError(error: unknown): State {
    return { error, crashed: true };
  }

  componentDidCatch(error: unknown, info: React.ErrorInfo) {
    // console.* is forwarded to the Rust logger, so this lands in the very
    // archive the error screen's export button produces
    console.error(
      `app crashed: ${describeError(error)}${info.componentStack ?? ''}`,
    );
    // once the tree is down, later unhandled rejections must escalate to the
    // error screen rather than a toast nobody can see
    markTreeDown();
  }

  render() {
    if (this.state.crashed) {
      return <FatalError error={this.state.error} />;
    }
    return this.props.children;
  }
}

export default ErrorBoundary;
