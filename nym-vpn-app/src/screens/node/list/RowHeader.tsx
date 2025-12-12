import { useCallback } from 'react';
import clsx from 'clsx';
import { Accordion } from '@base-ui-components/react';
import {
  SelectedKind,
  UiCountry,
  UiRegion,
  useNodeListState,
} from '../../../contexts';
import LocationInfo from './LocationInfo';
import FoldButton from './FoldButton';

export type RowHeaderProps = {
  hop: 'entry' | 'exit';
  isSelected: SelectedKind;
  node: UiCountry | UiRegion;
  onClick: (node: UiCountry | UiRegion) => void;
  sub?: boolean;
  gwCount: number;
  i18n: string;
};

function RowHeader({
  isSelected,
  hop,
  onClick,
  node,
  gwCount,
  i18n,
  sub,
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
        'flex flex-row justify-between rounded-r-lg',
        !sub
          ? ' bg-white dark:bg-charcoal'
          : 'bg-gainsboro dark:bg-charcoal/60',
        !sub
          ? 'hover:bg-white/60 dark:hover:bg-charcoal/85'
          : 'hover:bg-nordic-breeze hover:dark:bg-charcoal/75',
      )}
    >
      <div
        className={clsx(
          'w-1.5 rounded-r-sm',
          (isSelected === hop || isSelected === 'entry-and-exit') &&
            'bg-malachite',
          isSelected && isSelected !== hop && 'bg-iron',
        )}
        data-selected={isSelected ? isSelected : 'none'}
      />
      <div
        className={clsx('grow overflow-hidden truncate py-2')}
        onClick={() => onClick(node)}
      >
        {node.nodeType === 'country' ? (
          <LocationInfo node={node} name={i18n} gwCount={gwCount} />
        ) : (
          <LocationInfo node={node} name={node.name} gwCount={gwCount} />
        )}
      </div>
      <Accordion.Header className="flex p-2 items-center justify-center">
        <Accordion.Trigger
          render={(props, state) => <FoldButton html={props} state={state} />}
        />
      </Accordion.Header>
    </div>
  );
}

export default RowHeader;
