import clsx from 'clsx';
import { useClipboard } from '../../../../hooks';
import { ButtonIconNew } from '../../../../ui';

type ProxyUrlProps = {
  value: string;
  title: string;
  borderBottom?: boolean;
};

function ProxyUrl({ value, title, borderBottom = true }: ProxyUrlProps) {
  const { copy } = useClipboard();

  return (
    <div
      className={clsx(
        'flex flex-col',
        borderBottom &&
          'border-text-tertiary dark:border-text-secondary border-b py-2',
      )}
    >
      <p className="text-xs">{title}</p>
      <div className="flex items-center justify-between gap-4">
        <p className="text-text-secondary font-mono text-sm">{value}</p>
        <ButtonIconNew
          icon="content_copy"
          onClick={() => copy(value, false)}
          clickFeedback
          size="small"
        />
      </div>
    </div>
  );
}

export default ProxyUrl;
