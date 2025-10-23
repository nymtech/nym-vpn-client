import clsx from 'clsx';
import { Accordion } from '@base-ui-components/react';
import { SelectedKind, UiCountry, UiRegion } from '../../../contexts';
import CountryInfo from './CountryInfo';
import FoldButton from './FoldButton';

export type RowHeaderProps = {
  hop: 'entry' | 'exit';
  isSelected: SelectedKind;
  node: UiCountry | UiRegion;
  onClick: (node: UiCountry | UiRegion) => void;
  sub?: boolean;
  gwCount: number;
  i18n?: string;
};

function RowHeader({
  isSelected,
  hop,
  onClick,
  node,
  gwCount,
  i18n,
}: RowHeaderProps) {
  return (
    <div
      className={clsx(
        'flex flex-row justify-between',
        ' bg-white dark:bg-charcoal',
        'hover:bg-white/60 dark:hover:bg-charcoal/85',
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
          <CountryInfo country={node} name={i18n!} gwCount={gwCount} />
        ) : (
          <div>{`${node.name} - gw#${gwCount}`}</div>
        )}
      </div>
      <Accordion.Header className="flex py-2">
        <Accordion.Trigger
          render={(props, state) => <FoldButton html={props} state={state} />}
        />
      </Accordion.Header>
    </div>
  );
}

export default RowHeader;
