import { useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { ButtonNew, FlagIcon, MsIcon, type countryCode } from '../../ui';

type FoldState = 0 | 1 | 2;
type VpnMode = 'fast' | 'anonymous';

type NodeData = { code?: countryCode; name: string; location: string };

const DURATION = 0.3;

const EXIT_NODE: NodeData = {
  code: 'hu',
  name: 'hu-freedom-fight-mixnet',
  location: 'Budapest, Hungary',
};
const ENTRY_NODE: NodeData = {
  code: 'pl',
  name: 'pl-bober-bober-nodersowi',
  location: 'Warsaw, Poland',
};
const DEMO_NODE: NodeData = {
  code: 'ch',
  name: '169.128.6.931',
  location: 'Zurich, Switzerland',
};

const INITIAL_NODE: NodeData = {
  name: 'Best server for my location',
  location: 'Searching best location',
};

type ChevronProps = { onUp?: () => void; onDown?: () => void };

function Chevrons({ onUp, onDown }: ChevronProps) {
  if (!onUp && !onDown) return null;
  return (
    <div className="flex flex-col items-center shrink-0">
      {onUp && (
        <button
          type="button"
          onClick={onUp}
          className="text-secondary hover:text-white transition-colors cursor-default leading-none"
        >
          <MsIcon icon="keyboard_arrow_up" className="text-xl! leading-none" />
        </button>
      )}
      {onDown && (
        <button
          type="button"
          onClick={onDown}
          className="text-secondary hover:text-white transition-colors cursor-default leading-none"
        >
          <MsIcon
            icon="keyboard_arrow_down"
            className="text-xl! leading-none"
          />
        </button>
      )}
    </div>
  );
}

type NodeRowProps = NodeData & ChevronProps & { label?: string };

function NodeRow({ code, name, location, label, onUp, onDown }: NodeRowProps) {
  return (
    <div className="flex flex-col">
      {label && (
        <p className="text-secondary text-xs leading-5 tracking-[0.18px]">
          {label}
        </p>
      )}
      <div className="flex items-center gap-4">
        <MsIcon
          icon="signal_cellular_alt"
          className="text-[#5f6368] shrink-0"
        />
        {code && <FlagIcon code={code} alt={location} />}
        <span className="flex-1 min-w-0 text-white text-base leading-6 tracking-[-0.08px] truncate">
          {name}
        </span>
        {/* <MsIcon icon="passkey" className="text-malachite-200 shrink-0" />
        <MsIcon icon="hub" className="text-malachite-200 shrink-0" /> */}
        <Chevrons onUp={onUp} onDown={onDown} />
      </div>
      <p className="ml-10 text-secondary text-xs leading-5 tracking-[0.18px]">
        {location}
      </p>
    </div>
  );
}

type ModeToggleProps = ChevronProps & {
  activeMode: VpnMode;
  onToggle: () => void;
};

function ModeToggle({ activeMode, onToggle, onUp, onDown }: ModeToggleProps) {
  const isFast = activeMode === 'fast';
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-4 flex-1 min-w-0 justify-center">
        <button
          type="button"
          onClick={onToggle}
          className={clsx(
            'text-sm leading-[22px] tracking-[0.07px] w-20 text-right shrink-0 cursor-default transition-colors',
            isFast
              ? 'font-bold text-malachite-200'
              : 'text-secondary hover:text-white',
          )}
        >
          Fast
        </button>

        {/* Toggle pill */}
        <button
          type="button"
          onClick={onToggle}
          aria-label="Toggle VPN mode"
          className="relative bg-[#090909] h-10 w-20 rounded-full shrink-0 cursor-default"
        >
          <motion.div
            className="absolute top-[6px] bg-charcoal size-7 rounded-full flex items-center justify-center pointer-events-none"
            animate={{
              x: isFast ? 6 : 40,
              // backgroundColor: 'black',
              // backgroundColor: isFast ? '#d8d8d8' : '#182536',
            }}
            initial={false}
            transition={{ type: 'spring', stiffness: 420, damping: 32 }}
            // style={{ left: 6, right: 6 }}
          >
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={isFast ? 'electric_bolt' : 'visibility_off'}
                initial={{ opacity: 0, rotateX: 90 }}
                animate={{ opacity: 1, rotateX: 0 }}
                exit={{ opacity: 0, rotateX: -90 }}
                transition={{ duration: 0.1 }}
                className={clsx([
                  'font-icon text-2xl select-none inline-block rtl:-scale-x-100',
                  'shrink-0 text-xl!',
                  'text-malachite-200',
                  '[text-shadow:1px_1px_10px_#fff,1px_1px_10px_#ccc]',
                ])}
              >
                {isFast ? 'electric_bolt' : 'visibility_off'}
              </motion.span>
            </AnimatePresence>
          </motion.div>
        </button>

        <button
          type="button"
          onClick={onToggle}
          className={clsx(
            'text-sm leading-[22px] tracking-[0.07px] w-20 shrink-0 cursor-default transition-colors',
            !isFast
              ? 'font-bold text-malachite-200'
              : // ? 'font-bold text-[#a3cdff]'
                'text-secondary hover:text-white',
          )}
        >
          Anonymous
        </button>
      </div>
      <Chevrons onUp={onUp} onDown={onDown} />
    </div>
  );
}

const easeOutQuart = [0.22, 1, 0.36, 1] as const;

export function NewBottomComponent() {
  const [foldState, setFoldState] = useState<FoldState>(0);
  const [vpnMode, setVpnMode] = useState<VpnMode>('fast');

  const expand = () => setFoldState((s) => Math.min(s + 1, 2) as FoldState);
  const collapse = () => setFoldState((s) => Math.max(s - 1, 0) as FoldState);
  const toggleMode = () =>
    setVpnMode((m) => (m === 'fast' ? 'anonymous' : 'fast'));

  return (
    // No layout on the root — layout animation was FLIP-scaling the whole column on
    // the first big height change (0→1), which looked like a "squeeze". Enter/height
    // animations on children are enough for motion here.
    <div className="flex flex-col">
      <p>fold state: {foldState}</p>
      {/* ── Toggle section ────────────────────────────────────────────────── */}
      {/* Slides up from below when entering states 1/2 */}
      <AnimatePresence initial={false}>
        {foldState > 0 && (
          <motion.div
            key="toggle-header"
            initial={{ y: '100%' }}
            animate={{ y: 0 }}
            exit={{ y: '100%' }}
            transition={{ duration: DURATION, ease: easeOutQuart }}
            className="z-10 bg-[#1d1d1f] rounded-t-2xl pt-4 px-4"
          >
            <ModeToggle
              activeMode={vpnMode}
              onToggle={toggleMode}
              onDown={collapse}
              onUp={expand}
            />
            <div className="h-px bg-[#3b3b3b] rounded-full w-full mt-4" />
          </motion.div>
        )}
      </AnimatePresence>
      {/* ── Toggle section ────────────────────────────────────────────────── */}

      {/* ── Main card ─────────────────────────────────────────────────────── */}
      {/* layout so card height change is animated, not instant */}
      <div
        // layout
        className={clsx(
          'z-20 bg-[#1d1d1f] rounded-2xl px-4 py-4 flex flex-col transition-all duration-300',
          foldState > 0 && 'rounded-t-none',
        )}
      >
        <div
          className={clsx([
            'relative flex flex-col mb-4',
            foldState === 2 && 'space-y-4',
          ])}
        >
          <AnimatePresence initial={false}>
            <motion.div>
              <NodeRow
                key="entry-node"
                {...ENTRY_NODE}
                // {...DEMO_NODE}
                // {...INITIAL_NODE}
                label={foldState > 0 ? 'Nym entry node' : undefined}
                onUp={foldState === 0 ? expand : undefined}
              />
            </motion.div>
            {foldState === 2 && (
              <motion.div
                key="exit-node"
                initial={{ opacity: 0, y: '100%', height: 0 }}
                animate={{ opacity: 1, y: 0, height: 'auto' }}
                exit={{ opacity: 0, y: '100%', height: 0 }}
                transition={{ duration: DURATION }}
              >
                <NodeRow key="exit-node" {...EXIT_NODE} label="Nym exit node" />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
        {/* ── Main card ─────────────────────────────────────────────────────── */}

        {/* Button ───────────────────────────────────────────────────────── */}
        <div className="z-10">
          <ButtonNew>Tap to connect</ButtonNew>
        </div>
        {/* Button ───────────────────────────────────────────────────────── */}
      </div>
    </div>
  );
}
