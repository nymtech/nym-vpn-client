import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { MsIcon } from '../../../ui';
import { ListView } from '../../../store/nodeListState';

const VIEWS = [
  { id: 'favorites', icon: 'star', label: 'favorites.tab-favorites' },
  { id: 'all', icon: 'format_list_bulleted', label: 'favorites.tab-all' },
] as const;

function ViewToggle({
  view,
  onChange,
}: {
  view: ListView;
  onChange: (view: ListView) => void;
}) {
  const { t } = useTranslation('node-location');

  return (
    <div
      className="bg-surface-bg relative flex items-center gap-2 rounded-full p-0.5"
      data-testid="node-list-view-toggle"
    >
      <div
        className="bg-surface-elev absolute inset-y-0.5 w-[calc(50%-0.375rem)] rounded-full transition-[left] duration-300 ease-out"
        style={{
          left: view === 'favorites' ? '0.125rem' : 'calc(50% + 0.25rem)',
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
              'relative z-10 flex flex-1 cursor-default items-center justify-center gap-1.5 rounded-full px-4.5 py-2.5 text-sm font-bold transition-colors',
              isSelected
                ? 'text-primary'
                : 'text-text-secondary hover:bg-surface-elev',
            )}
          >
            <MsIcon
              icon={item.icon}
              filled={item.id === 'favorites' && isSelected}
              className="h-4 w-auto text-base! leading-none"
            />
            <span>{t(item.label)}</span>
          </button>
        );
      })}
    </div>
  );
}

export default ViewToggle;
