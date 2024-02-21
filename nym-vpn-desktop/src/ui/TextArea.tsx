import clsx from 'clsx';

type TextAreaProps = {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  // The number of visible text lines
  rows?: number;
  resize?: 'none' | 'vertical' | 'horizontal' | 'both';
  spellCheck?: boolean;
};

function TextArea({
  value,
  onChange,
  rows = 2,
  spellCheck,
  resize,
  label,
}: TextAreaProps) {
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    onChange(e.target.value);
  };

  const getResizeClass = (resize: string) => {
    switch (resize) {
      case 'none':
        return 'resize-none';
      case 'vertical':
        return 'resize-y';
      case 'horizontal':
        return 'resize-x';
      case 'both':
        return 'resize';
      default:
        return 'resize';
    }
  };

  return (
    <div
      className={clsx([
        'w-full flex flex-row items-center mb-2',
        label && 'relative',
      ])}
    >
      <textarea
        id="passphrase"
        name="passphrase"
        value={value}
        aria-multiline={true}
        className={clsx([
          'text-base bg-blanc-nacre dark:bg-baltic-sea',
          'border-cement-feet dark:border-gun-powder border rounded-lg',
          'hover:ring-1 hover:border-black hover:dark:border-white',
          'focus:border-black focus:dark:border-white',
          'focus:outline-none focus:ring-2 ring-black dark:ring-white',
          'w-full flex flex-row justify-between items-center py-4 px-4',
          'text-baltic-sea dark:text-mercury-pinkish',
          'placeholder:text-cement-feet placeholder:dark:text-mercury-mist',
          resize && getResizeClass(resize),
          label && 'relative',
        ])}
        onChange={handleChange}
        rows={rows}
        spellCheck={spellCheck}
      />
      {label && (
        <div
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-mercury-pinkish',
            'bg-blanc-nacre dark:bg-baltic-sea text-xs',
          ])}
        >
          {label}
        </div>
      )}
    </div>
  );
}

export default TextArea;
