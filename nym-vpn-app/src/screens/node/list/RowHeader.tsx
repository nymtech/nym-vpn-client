import { useCallback } from 'react';
import clsx from 'clsx';
import { Collapsible } from '@base-ui-components/react';
import { SelectedKind, UiCountry, UiRegion } from '../../../types/node';
import { useNodeListState } from '../../../store/nodeListState';
import LocationInfo from './LocationInfo';
import FoldButton from './FoldButton';

export type RowHeaderProps = {
  hop: 'entry' | 'exit';
  isSelected: SelectedKind;
  node: UiCountry | UiRegion;
  onClick: (node: UiCountry | UiRegion) => void;
  sub?: boolean;
  i18n: string;
  open?: boolean;
};

function RowHeader({
  isSelected,
  hop,
  onClick,
  node,
  i18n,
  sub,
  open,
}: RowHeaderProps) {
  const { exit: exitNodeList, entry: entryNodeList } = useNodeListState();

  const focused =
    hop === 'entry' ? entryNodeList.focused : exitNodeList.focused;

  const scrollToRowRef = useCallback(
    (htmlElement: HTMLDivElement) => {
      if (!htmlElement) return;
      const isFocused =
        focused?.type === node.nodeType &&
        ((node.nodeType === 'country' && focused.key === node.code) ||
          (node.nodeType === 'region' && focused.key === node.name));

      if (isFocused) {
        htmlElement.scrollIntoView({
          behavior: 'smooth',
          block: 'start',
        });
      }
    },
    [focused, node],
  );

  return (
    <div
      ref={scrollToRowRef}
      className={clsx(
        'p-2',
        'flex flex-row items-center justify-between transition-all duration-100',
        sub
          ? 'bg-gainsboro dark:bg-aph-light/40 hover:bg-gainsboro/60 hover:dark:bg-aph-light/20'
          : 'dark:bg-aph-light dark:hover:bg-aph-light/75 bg-white hover:bg-white/60',
        !sub && !open && 'rounded-2xl',
        !sub && open && 'rounded-2xl rounded-b-none',
        open && 'rounded-b-none!',
        'group-last/region:rounded-b-2xl',
        isSelected && 'border-2',
        (isSelected === hop || isSelected === 'entry-and-exit') &&
          'border-primary-active',
        isSelected && isSelected !== hop && 'border-text-secondary',
      )}
    >
      <div
        className={clsx('grow truncate overflow-hidden py-2')}
        onClick={() => onClick(node)}
      >
        <LocationInfo
          node={node}
          name={node.nodeType === 'country' ? i18n : node.name}
          hideFlag={node.nodeType === 'region'}
        />
      </div>
      <Collapsible.Trigger
        render={(props, state) => <FoldButton html={props} state={state} />}
      />
    </div>
  );
}

export default RowHeader;
