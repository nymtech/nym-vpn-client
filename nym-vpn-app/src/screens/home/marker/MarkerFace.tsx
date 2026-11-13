import { useEffect, useState } from 'react';
import type { MarkerMode } from './markerState';

// The marker's face says the mode. dVPN: an entry and an exit block, linked
// (2 hops). Mixnet: the three mix layers as a 3x3 field, with the five hops
// lit (entry, one node per layer, exit); hollow nodes are the unused ones.
// While not protected, nothing is lit: dVPN shows only its outline, Mixnet
// only its nodes. While connected the face plays an illustrative route: a
// packet steps through the hops, and the Mixnet route changes every two
// passes. It illustrates how the network works; it is not a traffic meter.
// Drawn in an 80 x 80 box.

type Point = readonly [number, number];

const MIX_ROUTES: readonly (readonly Point[])[] = [
  [
    [8, 40],
    [24, 22],
    [40, 58],
    [56, 22],
    [72, 40],
  ],
  [
    [8, 40],
    [24, 40],
    [40, 22],
    [56, 40],
    [72, 40],
  ],
  [
    [8, 40],
    [24, 58],
    [40, 22],
    [56, 58],
    [72, 40],
  ],
  [
    [8, 40],
    [24, 40],
    [40, 58],
    [56, 40],
    [72, 40],
  ],
];
const LAYER_X = [24, 40, 56];
const LAYER_Y = [22, 40, 58];
const NODE_R = 3.5;

// rhythm: the spec's data line ticks (dVPN 80 ms, Mixnet 120 ms)
const DVPN_TICK_MS = 80;
const MIX_TICK_MS = 120;
const DVPN_TICKS_PER_HOP = 3; // entry, link, exit: 240 ms each
const MIX_TICKS_PER_HOP = 2; // 240 ms per hop
const MIX_PASS = 6; // five hops and one rest beat
const MIX_PASSES_PER_ROUTE = 2;
const DIM = 0.32;
const TRAIL = 0.6;

function usePrefersReducedMotion() {
  const [reduced, setReduced] = useState(
    () => window.matchMedia('(prefers-reduced-motion: reduce)').matches,
  );
  useEffect(() => {
    const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
    const onChange = () => setReduced(mq.matches);
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }, []);
  return reduced;
}

// counts ticks while running; holds still while the window is hidden
function useTick(running: boolean, ms: number) {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    if (!running) {
      setTick(0);
      return;
    }
    const id = setInterval(() => {
      if (!document.hidden) setTick((t) => t + 1);
    }, ms);
    return () => clearInterval(id);
  }, [running, ms]);
  return tick;
}

function Node({ x, y }: { x: number; y: number }) {
  return <circle cx={x} cy={y} r={NODE_R} fill="none" strokeWidth={1.5} />;
}

function DvpnFace({ lit, playing }: { lit: boolean; playing: boolean }) {
  const tick = useTick(playing, DVPN_TICK_MS);
  // 0 entry, 1 link, 2 exit; -1 when still
  const hop = playing ? Math.floor(tick / DVPN_TICKS_PER_HOP) % 3 : -1;
  const op = (i: number) => (hop < 0 || hop === i ? 1 : DIM);
  const fill = lit ? undefined : 'none';
  return (
    <>
      <line x1={31} y1={40} x2={49} y2={40} opacity={op(1)} />
      <rect x={13} y={31} width={18} height={18} fill={fill} opacity={op(0)} />
      <rect x={49} y={31} width={18} height={18} fill={fill} opacity={op(2)} />
    </>
  );
}

function MixnetFace({ lit, playing }: { lit: boolean; playing: boolean }) {
  const tick = useTick(playing, MIX_TICK_MS);
  const step = Math.floor(tick / MIX_TICKS_PER_HOP);
  const route = playing
    ? MIX_ROUTES[
        Math.floor(step / (MIX_PASS * MIX_PASSES_PER_ROUTE)) % MIX_ROUTES.length
      ]
    : MIX_ROUTES[0];
  // index of the lit hop; -1 when still
  const hop = playing ? step % MIX_PASS : -1;
  const hopOpacity = (i: number) =>
    hop < 0 ? 1 : i === hop ? 1 : i < hop ? TRAIL : DIM;
  const onRoute = (x: number, y: number) =>
    lit && route.some(([rx, ry]) => rx === x && ry === y);
  return (
    <>
      {lit && (
        <polyline
          points={route.map((p) => p.join(',')).join(' ')}
          fill="none"
          opacity={playing ? 0.4 : 1}
        />
      )}
      {!lit && <Node x={8} y={40} />}
      {!lit && <Node x={72} y={40} />}
      {LAYER_X.flatMap((x) =>
        LAYER_Y.filter((y) => !onRoute(x, y)).map((y) => (
          <Node key={`${x}-${y}`} x={x} y={y} />
        )),
      )}
      {lit &&
        route.map(([x, y], i) => (
          <rect
            key={`${x}-${y}`}
            x={x - 5}
            y={y - 5}
            width={10}
            height={10}
            opacity={hopOpacity(i)}
          />
        ))}
    </>
  );
}

export function MarkerFace({
  mode,
  size,
  lit,
  playing,
}: {
  mode: MarkerMode;
  size: number;
  // false while not protected: no packets, only the outline or the nodes
  lit: boolean;
  // true while connected: plays the illustrative route
  playing: boolean;
}) {
  const reduced = usePrefersReducedMotion();
  const play = lit && playing && !reduced;
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 80 80"
      aria-hidden="true"
      // unlit: the ring tracks' grey, so the face sits back with the rings
      className={lit ? 'text-text-primary' : undefined}
      style={lit ? undefined : { color: 'var(--nv-connection-arc-track)' }}
      fill="currentColor"
      stroke="currentColor"
      strokeWidth={2}
      data-testid={`marker-face-${mode}`}
      data-lit={lit}
      data-playing={play}
    >
      {mode === 'mixnet' ? (
        <MixnetFace lit={lit} playing={play} />
      ) : (
        <DvpnFace lit={lit} playing={play} />
      )}
    </svg>
  );
}
