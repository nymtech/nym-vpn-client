export type ProxyInfoMessageProps = {
  message: string;
};

function ProxyInfoMessage({ message }: ProxyInfoMessageProps) {
  return (
    <span className="text-iron dark:text-bombay text-sm p-4">{message}</span>
  );
}

export default ProxyInfoMessage;
