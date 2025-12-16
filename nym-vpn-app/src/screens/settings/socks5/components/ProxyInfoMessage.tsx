export type ProxyInfoMessageProps = {
  message: string;
};

function ProxyInfoMessage({ message }: ProxyInfoMessageProps) {
  return (
    <div className="flex items-start gap-2 p-4">
      <span className="text-iron dark:text-bombay text-sm">{message}</span>
    </div>
  );
}

export default ProxyInfoMessage;

