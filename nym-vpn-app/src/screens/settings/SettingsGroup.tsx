import clsx from 'clsx';
import { ReactNode } from 'react';
import { Description, Label, Radio, RadioGroup } from '@headlessui/react';

type Setting = {
  title: string;
  leadingIcon?: string;
  leadingComponent?: ReactNode;
  desc?: string | ReactNode;
  onClick?: () => void;
  trailingIcon?: string;
  trailing?: ReactNode;
  disabled?: boolean;
  className?: string;
  'data-testid'?: string;
};
function isSettingItem(item: Setting | undefined | boolean): item is Setting {
  return (
    item !== undefined &&
    item !== false &&
    (item as Setting).title !== undefined
  );
}

type Props = {
  settings: (Setting | undefined | boolean)[];
  className?: string;
  'data-testid'?: string;
};

function SettingsGroup({ settings, className, ...rest }: Props) {
  const items = settings.filter((v) => isSettingItem(v));

  return (
    <RadioGroup className={clsx([className])} {...rest}>
      {items.map((setting, index) => {
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
              index !== 0 && 'border-t border-faded-lavender dark:border-ash',
              index === items.length - 1 && 'rounded-b-lg',
              setting.desc ? 'py-2' : 'py-4',
              setting.disabled &&
                'opacity-50 pointer-events-none cursor-default!',
            ])}
          >
            <div
              role={setting.disabled ? 'none' : 'button'}
              className="flex flex-1 items-center justify-between gap-4 overflow-hidden cursor-default"
            >
              {!!setting.leadingIcon && (
                <span className="font-icon text-2xl select-none text-bombay">
                  {setting.leadingIcon}
                </span>
              )}
              {!!setting.leadingComponent && setting.leadingComponent}
              <div className="flex flex-col flex-1 justify-center min-w-4">
                <Label
                  as="div"
                  className="text-base text-baltic-sea dark:text-white select-none truncate"
                >
                  {setting.title}
                </Label>
                <Description
                  as="div"
                  className="text-sm text-iron dark:text-bombay select-none truncate"
                >
                  {typeof setting.desc === 'string' ? (
                    <span>{setting.desc}</span>
                  ) : (
                    setting.desc
                  )}
                </Description>
              </div>
              {setting.trailingIcon && (
                <span className="font-icon text-2xl select-none text-bombay">
                  {setting.trailingIcon}
                </span>
              )}
              {setting.trailing && <div>{setting.trailing}</div>}
            </div>
          </Radio>
        );
      })}
    </RadioGroup>
  );
}

export default SettingsGroup;
