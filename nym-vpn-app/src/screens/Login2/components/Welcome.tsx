import { ButtonNew } from '../../../ui';

type Props = {
  onSignup: () => void;
  onLogin: () => void;
};

export function Welcome({ onSignup, onLogin }: Props) {
  return (
    <div className="flex flex-col items-center gap-6 h-full justify-between">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-2xl font-medium tracking-tight text-baltic-sea dark:text-white">
          Welcome!
        </h1>
        <p className="text-sm text-bombay text-center w-[281px]">
          To the next generation of privacy infrastructure.
        </p>
      </div>
      <div className="flex flex-col gap-3 w-full">
        <ButtonNew onClick={onSignup}>Sign up</ButtonNew>
        <ButtonNew onClick={onLogin}>Login to my account</ButtonNew>
        <p className="text-xs text-bombay text-center leading-5">
          By continuing, you agree to{' '}
          <span className="font-semibold text-baltic-sea dark:text-white">
            our Terms
          </span>{' '}
          and acknowledge{' '}
          <span className="font-semibold text-baltic-sea dark:text-white">
            our Privacy Policy
          </span>
        </p>
      </div>
    </div>
  );
}
