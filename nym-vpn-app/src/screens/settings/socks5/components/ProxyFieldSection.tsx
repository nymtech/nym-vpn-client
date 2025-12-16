import { ButtonIcon, MsIcon, TextInput } from '../../../../ui';

export type ProxyFieldSectionProps = {
  title: string;
  icon?: string;
  value: string;
  onValueChange?: (value: string) => void;
  onCopy: () => void;
  disabled?: boolean;
  showInput?: boolean;
};

function ProxyFieldSection({
  title,
  icon = 'tag',
  value,
  onValueChange,
  onCopy,
  disabled = false,
  showInput = true,
}: ProxyFieldSectionProps) {
  return (
    <div className="flex flex-col gap-2 border-b border-bombay dark:border-baltic-sea p-4">
      <div className="flex items-center gap-2">
        <MsIcon icon={icon} className="text-iron dark:text-bombay text-2xl" />
        <p className="text-base font-medium">{title}</p>
      </div>
      <div className="flex items-center justify-between gap-4">
        {showInput && onValueChange ? (
          <TextInput
            onChange={onValueChange}
            disabled={disabled}
            value={value}
            color="default"
          />
        ) : (
          <p className="text-iron dark:text-bombay font-mono text-sm">
            {value}
          </p>
        )}
        <ButtonIcon
          icon="content_copy"
          color="chalk"
          onClick={onCopy}
          clickFeedback
          noDefaultSize
        />
      </div>
    </div>
  );
}

export default ProxyFieldSection;
