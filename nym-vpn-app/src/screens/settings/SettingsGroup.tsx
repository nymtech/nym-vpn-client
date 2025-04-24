import clsx from 'clsx';
import { ReactNode } from 'react';
import { Description, Label, Radio, RadioGroup } from '@headlessui/react';

type Setting = {
  title: string;
  leadingIcon?: string;
  desc?: string | ReactNode;
  onClick?: () => void;
  trailing?: ReactNode;
  disabled?: boolean;
  className?: string;
  'data-testid'?: string;
};

type Props = {
  settings: Setting[];
  className?: string;
  'data-testid'?: string;
};

function SettingsGroup({ settings, className, ...rest }: Props) {
  return (
    <RadioGroup className={clsx([className])} {...rest}>
      {settings.map((setting, index) => {
        const testId = setting['data-testid'];

        return (
          <Radio
            key={setting.title}
            value={setting.title}
            onClick={setting.onClick}
            className={clsx([
              'cursor-default',
              'bg-white dark:bg-charcoal relative flex px-5 py-2 focus:outline-hidden min-h-16',
              'hover:bg-white/60 dark:hover:bg-charcoal/85',
              'transition duration-75',
              index === 0 && 'rounded-t-lg',
              index === settings.length - 1 &&
                settings.length === 2 &&
                'border-t border-faded-lavender dark:border-ash',
              index !== 0 &&
                index !== settings.length - 1 &&
                'border-y border-faded-lavender dark:border-ash',
              index === settings.length - 1 && 'rounded-b-lg',
              setting.desc ? 'py-2' : 'py-4',
              setting.disabled &&
                'opacity-50 pointer-events-none cursor-default!',
            ])}
            data-testid={testId}
          >
            <div
              role={setting.disabled ? 'none' : 'button'}
              className="flex flex-1 items-center justify-between gap-4 overflow-hidden cursor-default"
            >
              {setting.leadingIcon && (
                <span
                  className="font-icon text-2xl select-none dark:text-white"
                  data-testid={testId ? `${testId}-icon` : undefined}
                >
                  {setting.leadingIcon}
                </span>
              )}
              <div className="flex flex-col flex-1 justify-center min-w-4">
                <Label
                  as="div"
                  className="text-base text-baltic-sea dark:text-white select-none truncate"
                  data-testid={testId ? `${testId}-title` : undefined}
                >
                  {setting.title}
                </Label>
                <Description
                  as="div"
                  className="text-sm text-iron dark:text-bombay select-none truncate"
                  data-testid={testId ? `${testId}-description` : undefined}
                >
                  {typeof setting.desc === 'string' ? (
                    <span>{setting.desc}</span>
                  ) : (
                    setting.desc
                  )}
                </Description>
              </div>
              {setting.trailing}
            </div>
          </Radio>
        );
      })}
    </RadioGroup>
  );
}

export default SettingsGroup;
