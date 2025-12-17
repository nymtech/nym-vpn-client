import { ReactNode } from 'react';
import MsIcon from '../../../ui/MsIcon';

export type ProxyInfoCardProps = {
  children: ReactNode;
  title: string;
};

function ProxyInfoCard({ children, title }: ProxyInfoCardProps) {
  return (
    <div className="bg-white dark:bg-charcoal rounded-lg flex flex-col gap-5 p-4">
      <div className="flex items-center gap-2">
        <MsIcon
          icon="tag"
          className="text-iron dark:text-bombay text-2xl leading-1"
        />
        <p className="text-base font-medium">{title}</p>
      </div>
      {children}
    </div>
  );
}

export default ProxyInfoCard;
