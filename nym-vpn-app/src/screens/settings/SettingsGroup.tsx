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
              'dark:bg-aph-light relative flex min-h-16 bg-white px-5 py-2 focus:outline-hidden',
              'dark:hover:bg-charcoal/85 hover:bg-white/60',
              'transition duration-75',
              index === 0 && 'rounded-t-2xl',
              index !== 0 && 'border-faded-lavender dark:border-ash border-t',
              index === items.length - 1 && 'rounded-b-2xl',
              setting.desc ? 'py-2' : 'py-4',
              setting.disabled &&
                'pointer-events-none cursor-default! opacity-50',
            ])}
          >
            <div
              role={setting.disabled ? 'none' : 'button'}
              className="flex flex-1 cursor-default items-center justify-between gap-4 overflow-hidden"
            >
              {!!setting.leadingIcon && (
                <span className="font-icon text-text-secondary text-2xl select-none">
                  {setting.leadingIcon}
                </span>
              )}
              {!!setting.leadingComponent && setting.leadingComponent}
              <div className="flex min-w-4 flex-1 flex-col justify-center">
                <Label
                  as="div"
                  className="text-text-primary truncate text-base select-none"
                >
                  {setting.title}
                </Label>
                <Description
                  as="div"
                  className="text-text-secondary truncate text-sm select-none"
                >
                  {typeof setting.desc === 'string' ? (
                    <span>{setting.desc}</span>
                  ) : (
                    setting.desc
                  )}
                </Description>
              </div>
              {setting.trailingIcon && (
                <span className="font-icon text-bombay text-xl select-none">
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
