import clsx from 'clsx';

// Loading component rendering transitions for lazy loaded route
function RouteLoading() {
  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-80',
        'bg-blanc-nacre dark:bg-baltic-sea',
      ])}
    >
      {/* Top-bar placeholder */}
      <div className="w-full h-16 shadow bg-white dark:bg-baltic-sea-jaguar"></div>
    </div>
  );
}

export default RouteLoading;
