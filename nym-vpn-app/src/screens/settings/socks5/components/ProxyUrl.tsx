import clsx from 'clsx';
import { useClipboard } from '../../../../hooks';
import { ButtonIcon } from '../../../../ui';

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
        borderBottom && 'border-b border-bombay dark:border-iron py-2',
      )}
    >
      <p className="text-xs">{title}</p>
      <div className="flex items-center justify-between gap-4">
        <p className="text-iron dark:text-bombay font-mono text-sm">{value}</p>
        <ButtonIcon
          clickFeedback
          noDefaultSize
          color="chalk"
          icon="content_copy"
          onClick={() => copy(value, false)}
        />
      </div>
    </div>
  );
}

export default ProxyUrl;
