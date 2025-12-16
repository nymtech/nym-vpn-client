import { ReactNode } from 'react';

export type ProxyInfoCardProps = {
  children: ReactNode;
};

function ProxyInfoCard({ children }: ProxyInfoCardProps) {
  return (
    <div className="bg-white dark:bg-charcoal rounded-xl flex flex-col gap-4">
      {children}
    </div>
  );
}

export default ProxyInfoCard;

