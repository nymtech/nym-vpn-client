import clsx from 'clsx';
import { ReactNode } from 'react';
import { Description, Label, Radio, RadioGroup } from '@headlessui/react';

type Setting = {
  title: string;
  titleTrailing?: ReactNode;
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
              'group cursor-default',
              'relative flex min-h-16 px-5 py-2 focus:outline-hidden',
              'bg-surface-elev hover:bg-surface-hair',
              'transition duration-75',
              index === 0 && 'rounded-t-2xl',
              index !== 0 && 'border-surface-bg border-t',
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
                <div className="flex items-center gap-2 overflow-hidden">
                  <Label
                    as="div"
                    className="text-text-primary truncate text-base select-none"
                  >
                    {setting.title}
                  </Label>
                  {setting.titleTrailing}
                </div>
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
                <span className="font-icon text-text-tertiary text-xl select-none">
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
