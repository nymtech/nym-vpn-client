import { Separator } from '@base-ui-components/react';
import { SettingsMenuCardBig } from '../../../ui';

export function PerformanceCard() {
  return (
    <SettingsMenuCardBig
      header={
        <div className="p-5 pb-0 w-full flex flex-row items-start justify-start">
          <p className=" text-left text-sm text-iron dark:text-bombay whitespace-pre-line">
            Expected performance based on current settings
          </p>
        </div>
      }
      footer={
        <div className="p-5 pt-0 w-full">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            Expected round-trip latency (RTT) on a standard network
          </p>
        </div>
      }
    >
      <div className="flex flex-col justify-center items-start gap-3">
        <DataRow label="Speed">
          <div className="flex gap-1 items-center overflow-hidden select-none">
            <p className="text-malachite-moss dark:text-malachite font-medium">
              Upt to 1 Mbps
            </p>
          </div>
        </DataRow>
        <Separator
          orientation="horizontal"
          className="w-full h-px bg-bombay dark:bg-iron"
        />
        <DataRow label="Privacy">
          <div className="flex gap-1 items-center overflow-hidden select-none">
            <p className="text-malachite-moss dark:text-malachite font-medium">
              At least 700ms
            </p>
          </div>
        </DataRow>
      </div>
    </SettingsMenuCardBig>
  );
}

const DataRow = ({
  children,
  label,
}: {
  children: React.ReactNode;
  label: string;
}) => (
  <div className="w-full flex justify-between items-center">
    <p className="text-iron dark:text-bombay truncate select-none">{label}</p>
    <div className="flex flex-nowrap items-center gap-2 overflow-hidden">
      {children}
    </div>
  </div>
);
