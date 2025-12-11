import { type ReactNode } from 'react';
import { ButtonIcon, type DraggableListItem } from '../../../ui';

export type DnsItem = DraggableListItem & {
  dns: string;
};

export function DnsItemContent({
  item,
  dragHandle,
  onDelete,
}: {
  item: DnsItem;
  dragHandle: ReactNode;
  onDelete: (dns: string) => void;
}) {
  return (
    <div className="flex flex-row items-center justify-between gap-2 p-3 pl-">
      <div className="flex flex-row items-center gap-2 flex-1 min-w-0">
        {dragHandle}
        <p className="text-base text-baltic-sea dark:text-white truncate">
          {item.dns}
        </p>
      </div>
      <ButtonIcon
        icon="delete_outline"
        color="chalk"
        onClick={() => {
          onDelete(item.id);
        }}
        noDefaultSize
        className="shrink-0"
      />
    </div>
  );
}
