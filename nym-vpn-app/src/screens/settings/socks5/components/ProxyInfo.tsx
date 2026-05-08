type ProxyInfoProps = {
  text: string;
};

function ProxyInfo({ text }: ProxyInfoProps) {
  return <span className="text-text-secondary text-sm">{text}</span>;
}

export default ProxyInfo;
