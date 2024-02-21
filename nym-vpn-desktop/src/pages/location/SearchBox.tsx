import clsx from 'clsx';
import { InputEvent } from '../../types';
import { MsIcon } from '../../ui';

interface SearchProps {
  value: string;
  onChange: (e: InputEvent) => void;
  placeholder: string;
}

/* eslint-disable jsx-a11y/no-autofocus */
export default function SearchBox({
  value,
  onChange,
  placeholder,
}: SearchProps) {
  return (
    <div className="w-full relative flex flex-row items-center px-4 mb-2">
      <input
        type="text"
        id="country_search"
        value={value}
        className={clsx([
          'bg-blanc-nacre dark:bg-baltic-sea focus:outline-none focus:ring-0',
          'w-full flex flex-row justify-between items-center py-3 px-4 pl-11',
          'text-baltic-sea dark:text-mercury-pinkish',
          'placeholder:text-cement-feet placeholder:dark:text-mercury-mist',
          'border-cement-feet dark:border-gun-powder border rounded-lg',
          'relative text-base',
        ])}
        placeholder={placeholder}
        onChange={onChange}
        autoFocus
      />
      <div
        className={clsx([
          'absolute left-7 -top-2 px-1',
          'text-cement-feet dark:text-mercury-mist',
          'bg-blanc-nacre dark:bg-baltic-sea text-xs',
        ])}
      >
        Search
      </div>
      <MsIcon
        icon="search"
        style="absolute left-7 text-baltic-sea dark:text-laughing-jack"
      />
    </div>
  );
}
