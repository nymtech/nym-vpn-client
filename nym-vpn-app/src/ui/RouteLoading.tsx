import clsx from 'clsx';

// Loading component rendering transitions for lazy loaded route
function RouteLoading() {
  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-80',
        'bg-faded-lavender dark:bg-ash',
      ])}
    >
      {/* Top-bar placeholder */}
      <div className="w-full h-16 shadow-sm bg-white dark:bg-octave-arsenic"></div>
    </div>
  );
}

export default RouteLoading;
