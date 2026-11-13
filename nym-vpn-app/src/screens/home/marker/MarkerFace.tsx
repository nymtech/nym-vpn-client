import type { MarkerMode } from './markerState';

// The marker's face says the mode. dVPN: an entry and an exit block, linked
// (2 hops). Mixnet: the three mix layers as a 3x3 field, with the five hops
// lit (entry, one node per layer, exit); hollow nodes are the unused ones.
// While not protected, nothing is lit: dVPN shows only its outline, Mixnet
// only its nodes. Drawn in an 80 x 80 box.

type Point = readonly [number, number];

// one route through the mix layers; PR 4 cycles routes with the data line
const MIX_ROUTE: readonly Point[] = [
  [8, 40],
  [24, 22],
  [40, 58],
  [56, 22],
  [72, 40],
];
const LAYER_X = [24, 40, 56];
const LAYER_Y = [22, 40, 58];
const NODE_R = 3.5;

function Node({ x, y }: { x: number; y: number }) {
  return <circle cx={x} cy={y} r={NODE_R} fill="none" strokeWidth={1.5} />;
}

function DvpnFace({ lit }: { lit: boolean }) {
  const fill = lit ? undefined : 'none';
  return (
    <>
      <line x1={31} y1={40} x2={49} y2={40} />
      <rect x={13} y={31} width={18} height={18} fill={fill} />
      <rect x={49} y={31} width={18} height={18} fill={fill} />
    </>
  );
}

function MixnetFace({ lit }: { lit: boolean }) {
  const onRoute = (x: number, y: number) =>
    lit && MIX_ROUTE.some(([rx, ry]) => rx === x && ry === y);
  return (
    <>
      {lit && (
        <polyline
          points={MIX_ROUTE.map((p) => p.join(',')).join(' ')}
          fill="none"
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
        MIX_ROUTE.map(([x, y]) => (
          <rect key={`${x}-${y}`} x={x - 5} y={y - 5} width={10} height={10} />
        ))}
    </>
  );
}

export function MarkerFace({
  mode,
  size,
  lit,
}: {
  mode: MarkerMode;
  size: number;
  // false while not protected: no packets, only the outline or the nodes
  lit: boolean;
}) {
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
    >
      {mode === 'mixnet' ? <MixnetFace lit={lit} /> : <DvpnFace lit={lit} />}
    </svg>
  );
}
