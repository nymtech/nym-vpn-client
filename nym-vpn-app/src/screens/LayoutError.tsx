import { useRouteError } from 'react-router';
import FatalError from './FatalError';

// Error element for the layout routes. Once `MainLayout` itself has thrown
// there is no top bar or background left to render into, so this takes the
// whole window like a crash caught by the error boundary would. Screen-level
// errors are handled one level down by `Error`, which keeps the layout.
function LayoutError() {
  return <FatalError error={useRouteError()} />;
}

export default LayoutError;
