import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { MsIcon } from '../../../ui';
import { ListView } from '../../../store/nodeListState';

const VIEWS = [
  { id: 'favorites', icon: 'star', label: 'favorites.tab-favorites' },
  { id: 'recents', icon: 'history', label: 'favorites.tab-recents' },
  { id: 'all', icon: 'format_list_bulleted', label: 'favorites.tab-all' },
] as const;

// Must match the container's `gap-2` and `p-0.5` for the indicator to line up
// with the buttons it tracks.
const GAP = '0.5rem';
const PADDING = '0.25rem'; // 0.125rem on each side

function ViewToggle({
  view,
  onChange,
}: {
  view: ListView;
  onChange: (view: ListView) => void;
}) {
  const { t } = useTranslation('node-location');

  const selectedIndex = Math.max(
    0,
    VIEWS.findIndex((item) => item.id === view),
  );

  return (
    <div
      className="bg-surface-bg relative flex items-center gap-2 rounded-full p-0.5"
      data-testid="node-list-view-toggle"
    >
      {/*
        The buttons are `flex-1` and `min-w-0`, so each is exactly
        (row - padding - gaps) / count wide. The `min-w-0` is load-bearing: a
        flex item otherwise refuses to shrink below its own label, and buttons
        wider than an equal share overflow the row and leave this indicator —
        which is sized off the container — pointing between them.

        The percentage in `translateX` resolves against the indicator's own width
        rather than the container's, so stepping by that width plus one gap lands
        it on the nth button at any count.
      */}
      <div
        className="bg-surface-elev absolute inset-y-0.5 left-0.5 rounded-full transition-transform duration-300 ease-out"
        style={{
          width: `calc((100% - ${PADDING} - ${GAP} * ${VIEWS.length - 1}) / ${VIEWS.length})`,
          transform: `translateX(calc(${selectedIndex} * (100% + ${GAP})))`,
        }}
      />
      {VIEWS.map((item) => {
        const isSelected = view === item.id;
        return (
          <button
            key={item.id}
            type="button"
            onClick={() => onChange(item.id)}
            aria-pressed={isSelected}
            data-testid={`node-list-view-${item.id}`}
            className={clsx(
              'relative z-10 flex min-w-0 flex-1 cursor-default items-center justify-center gap-1.5 rounded-full px-2 py-2.5 text-sm font-bold transition-colors',
              isSelected
                ? 'text-primary'
                : 'text-text-secondary hover:bg-surface-elev',
            )}
          >
            <MsIcon
              icon={item.icon}
              filled={item.id === 'favorites' && isSelected}
              className="h-4 w-auto shrink-0 text-base! leading-none"
            />
            <span className="truncate">{t(item.label)}</span>
          </button>
        );
      })}
    </div>
  );
}

export default ViewToggle;
