import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { DotLottie, DotLottieReact } from '@lottiefiles/dotlottie-react';
import { useMainDispatch, useMainState } from '../contexts';
import { StateDispatch } from '../types';

function IntroAnim() {
  const [dotLottie, setDotLottie] = useState<DotLottie | null>(null);
  const [completed, setCompleted] = useState(false);

  const { uiTheme } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  useEffect(() => {
    console.log('___SPLASH ANIM INIT');
    const onComplete = () => {
      console.log('___SPLASH ANIM DONE');
      setCompleted(true);
    };

    // Listen to events emitted by the DotLottie instance when it is available.
    if (dotLottie) {
      dotLottie.addEventListener('complete', onComplete);
      dotLottie.addEventListener('frame', onFrameChange);
    }

    function onFrameChange(args: any) {
      console.log(args);
      if (args?.currentFrame > 27) {
        setCompleted(true);
      }
    }

    return () => {
      // Remove event listeners when the component is unmounted.
      if (dotLottie) {
        dotLottie.removeEventListener('complete', onComplete);
        dotLottie.removeEventListener('frame', onFrameChange);
      }
    };
  }, [dotLottie, dispatch]);

  const dotLottieRefCallback = (anim: DotLottie) => {
    setDotLottie(anim);
  };

  if (completed) {
    return null;
  }
  console.log('___SPLASH ANIM');

  return (
    <div className={clsx([uiTheme === 'dark' && 'dark'])}>
      <div
        className={clsx([
          'h-full w-full absolute z-200 flex flex-col min-w-64',
          'bg-faded-lavender dark:bg-ash overflow-hidden',
        ])}
      >
        <DotLottieReact
          // src="https://lottie.host/63e43fb7-61be-486f-aef2-622b144f7fc1/2m8UGcP8KR.json"
          src="/animations/splash.json"
          dotLottieRefCallback={dotLottieRefCallback}
          autoplay
          loop={false}
        />
      </div>
    </div>
  );
}

export default IntroAnim;
