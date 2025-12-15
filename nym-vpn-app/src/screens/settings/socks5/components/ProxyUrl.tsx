import { useClipboard } from '../../../../hooks';
import { ButtonIcon } from '../../../../ui';

type ProxyUrlProps = {
  value: string;
  title: string;
};

function ProxyUrl({ value, title }: ProxyUrlProps) {
  const { copy } = useClipboard();

  return (
    <div className="flex flex-col">
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
