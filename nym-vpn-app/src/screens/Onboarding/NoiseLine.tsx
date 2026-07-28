import { DotLottieReact } from '@lottiefiles/dotlottie-react';
import { useAppStore } from '../../store';

const WIDTH = 514;
const HEIGHT = 39;

export function NoiseLine() {
  const uiTheme = useAppStore((s) => s.uiTheme);

  return (
    <div className="flex w-full justify-center overflow-hidden">
      <div style={{ width: WIDTH, height: HEIGHT }} className="shrink-0">
        <DotLottieReact
          src={
            uiTheme === 'dark'
              ? '/animations/noise-line.json'
              : '/animations/noise-line-light.json'
          }
          autoplay
          loop
        />
      </div>
    </div>
  );
}
