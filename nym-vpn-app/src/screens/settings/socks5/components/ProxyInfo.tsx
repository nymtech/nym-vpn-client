type ProxyInfoProps = {
  text: string;
};

function ProxyInfo({ text }: ProxyInfoProps) {
  return <span className="text-iron dark:text-bombay text-sm">{text}</span>;
}

export default ProxyInfo;
