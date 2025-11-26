import useCustomDns from '../../../hooks/useCustomDns';
import { MsIcon } from '../../../ui';

export function DefaultDnsServers() {
  const { defaultDns } = useCustomDns();

  return (
    <div className="flex flex-col">
      <p className="text-xs">Default DNS servers</p>
      <div className="py-3">
        {defaultDns.map((dns) => (
          <div
            key={dns}
            className="flex flex-row items-center gap-2 p-3 pl-0 border-t last:border-b border-bombay dark:border-iron"
          >
            <MsIcon icon="dns" className="text-iron dark:text-bombay" />
            <p className="text-base text-baltic-sea dark:text-white truncate">
              {dns}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
