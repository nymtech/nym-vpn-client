import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { DotLottie, DotLottieReact } from '@lottiefiles/dotlottie-react';
import { S_STATE } from '../static';

function IntroAnim() {
  const [dotLottie, setDotLottie] = useState<DotLottie | null>(null);
  const [completed, setCompleted] = useState(false);

  useEffect(() => {
    console.log('___SPLASH ANIM INIT');
    const onComplete = () => {
      console.log('___SPLASH ANIM DONE');
      setCompleted(true);
    };

    if (dotLottie) {
      dotLottie.addEventListener('complete', onComplete);
    }

    return () => {
      if (dotLottie) {
        dotLottie.removeEventListener('complete', onComplete);
      }
    };
  }, [dotLottie]);

  const dotLottieRefCallback = (anim: DotLottie) => {
    setDotLottie(anim);
  };

  return (
    <div className={clsx([S_STATE.uiTheme === 'dark' && 'dark'])}>
      <AnimatePresence>
        {!completed && (
          <motion.div
            className={clsx([
              'h-full w-full absolute z-200 flex flex-col items-center min-w-44',
              'bg-faded-lavender dark:bg-ash overflow-hidden',
            ])}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.1, ease: 'easeOut' }}
          >
            <DotLottieReact
              src="https://lottie.host/63e43fb7-61be-486f-aef2-622b144f7fc1/2m8UGcP8KR.json"
              // src="/animations/splash.json"
              dotLottieRefCallback={dotLottieRefCallback}
              autoplay
              loop={false}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default IntroAnim;
