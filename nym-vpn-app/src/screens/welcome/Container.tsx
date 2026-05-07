import clsx from 'clsx';
import { AnimatePresence, Variants, motion } from 'motion/react';
import { useRef, useState } from 'react';
import { NymVpnTextLogo } from '../../assets';
import { ButtonIconNew } from '../../ui';
import { useAppStore } from '../../store';
import { InteractiveCard } from '../home/InteractiveCard';
import { Welcome } from './components/Welcome';
import { Signup } from './components/Signup';
import { Login } from './components/Login';
import { PassphraseEnter } from './components/PassphraseEnter';

type View = 'welcome' | 'signup' | 'login' | 'passphrase';

// 1 = forward, -1 = backward
// enter: forward → from right (+100%), backward → from left (-100%)
// exit:  forward → to left (−100%),   backward → to right (+100%)
const slideVariants: Variants = {
  enter: (dir: number) => ({ x: dir > 0 ? '100%' : '-100%' }),
  visible: { x: 0 },
  exit: (dir: number) => ({ x: dir > 0 ? '-100%' : '100%' }),
};

const backActions: Partial<Record<View, { target: View; label: string }>> = {
  signup: { target: 'welcome', label: 'Back to welcome' },
  login: { target: 'welcome', label: 'Back to welcome' },
  passphrase: { target: 'login', label: 'Back to login' },
};

function WelcomeScreenContainer() {
  const [view, setView] = useState<View>('welcome');
  const [dir, setDir] = useState(1);
  const hasNavigated = useRef(false);
  const uiTheme = useAppStore((s) => s.uiTheme);

  const navigate = (to: View, direction: 1 | -1 = 1) => {
    hasNavigated.current = true;
    setDir(direction);
    setView(to);
  };

  const backAction = backActions[view];

  return (
    <InteractiveCard className="min-h-96">
      {/* Static header — never animates on navigation */}
      <div className="mb-12">
        <div className="flex items-center justify-center relative h-[27px]">
          {backAction && (
            <ButtonIconNew
              initialAnimation={true}
              icon="arrow_back"
              onClick={() => navigate(backAction.target, -1)}
              className="absolute left-0 text-bombay hover:text-baltic-sea dark:hover:text-white transition-noborder cursor-default"
            />
          )}
          <NymVpnTextLogo
            className={clsx(
              'w-[100px] h-[27px]',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
        </div>
      </div>

      {/* Animated content area */}
      <div className="overflow-hidden flex-1">
        <AnimatePresence mode="wait" custom={dir}>
          <motion.div
            key={view}
            custom={dir}
            variants={slideVariants}
            // Skip enter animation on first load — the card slide-up carries the content in
            initial={hasNavigated.current ? 'enter' : false}
            animate="visible"
            exit="exit"
            transition={{ duration: 0.28, ease: 'easeInOut' }}
            className="h-full"
          >
            {view === 'welcome' && (
              <Welcome
                onSignup={() => navigate('signup', 1)}
                onLogin={() => navigate('login', 1)}
              />
            )}
            {view === 'signup' && <Signup />}
            {view === 'login' && (
              <Login onPassphrase={() => navigate('passphrase', 1)} />
            )}
            {view === 'passphrase' && <PassphraseEnter />}
          </motion.div>
        </AnimatePresence>
      </div>
    </InteractiveCard>
  );
}

export default WelcomeScreenContainer;
