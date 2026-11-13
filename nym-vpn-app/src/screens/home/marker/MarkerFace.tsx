import type { MarkerMode } from './markerState';

// The marker's face says the mode. dVPN: an entry and an exit block, linked
// (2 hops). Mixnet: the three mix layers as a 3x3 field, with the five hops
// lit (entry, one node per layer, exit); hollow nodes are the unused ones.
// Drawn in an 80 x 80 box.

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

function DvpnFace() {
  return (
    <>
      <line x1={31} y1={40} x2={49} y2={40} />
      <rect x={13} y={31} width={18} height={18} />
      <rect x={49} y={31} width={18} height={18} />
    </>
  );
}

function MixnetFace() {
  const used = (x: number, y: number) =>
    MIX_ROUTE.some(([rx, ry]) => rx === x && ry === y);
  return (
    <>
      <polyline
        points={MIX_ROUTE.map((p) => p.join(',')).join(' ')}
        fill="none"
      />
      {LAYER_X.flatMap((x) =>
        LAYER_Y.filter((y) => !used(x, y)).map((y) => (
          <circle
            key={`${x}-${y}`}
            cx={x}
            cy={y}
            r={3.5}
            fill="none"
            strokeWidth={1.5}
          />
        )),
      )}
      {MIX_ROUTE.map(([x, y]) => (
        <rect key={`${x}-${y}`} x={x - 5} y={y - 5} width={10} height={10} />
      ))}
    </>
  );
}

export function MarkerFace({ mode, size }: { mode: MarkerMode; size: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 80 80"
      aria-hidden="true"
      className="text-text-primary"
      fill="currentColor"
      stroke="currentColor"
      strokeWidth={2}
      data-testid={`marker-face-${mode}`}
    >
      {mode === 'mixnet' ? <MixnetFace /> : <DvpnFace />}
    </svg>
  );
}
