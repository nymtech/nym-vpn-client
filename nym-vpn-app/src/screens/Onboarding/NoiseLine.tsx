const WIDTH = 514;
const HEIGHT = 39;
const CELL_HEIGHT = 13;

// Each state is a contiguous run of cells laid out left to right, every cell
// `w` wide; `rows` holds the vertical slot (0, 1 or 2) of each cell. Every
// state is held for 200ms by `.noise-line-state`, staggered by its index into
// a 1s loop, so exactly one is visible at a time.
const STATES = [
  {
    w: 13.5455,
    rows: [
      0, 1, 1, 1, 2, 1, 0, 2, 1, 1, 2, 2, 2, 0, 1, 2, 0, 1, 1, 0, 0, 2, 2, 1, 1,
      2, 1, 1, 1, 2, 1, 1, 0,
    ],
  },
  {
    w: 13,
    rows: [
      1, 0, 2, 1, 2, 1, 2, 0, 2, 1, 0, 0, 2, 2, 1, 1, 2, 1, 1, 0, 2, 1, 2, 1, 2,
      1, 0, 0, 0, 2, 2, 2, 1,
    ],
  },
  {
    w: 13,
    rows: [
      1, 1, 2, 2, 0, 1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 0, 0, 2, 1, 0, 1, 2, 0, 1, 0,
      0, 1, 0, 2, 0, 1, 1, 2, 1, 1, 1,
    ],
  },
  {
    w: 13,
    rows: [
      2, 2, 1, 1, 2, 0, 0, 1, 2, 1, 0, 1, 2, 1, 1, 0, 0, 0, 1, 2, 1, 1, 1, 1, 2,
      2, 1, 1, 0, 2, 2, 1, 0, 0, 0, 0, 2, 2, 1,
    ],
  },
  {
    w: 13.5263,
    rows: [
      2, 1, 2, 2, 0, 1, 1, 0, 2, 1, 2, 1, 1, 0, 1, 1, 2, 1, 1, 0, 2, 0, 2, 1, 0,
      0, 2, 0, 1, 0, 1, 0, 0, 1, 2, 0, 1, 0,
    ],
  },
];

export function NoiseLine() {
  return (
    <div className="flex w-full justify-center overflow-hidden">
      <svg
        width={WIDTH}
        height={HEIGHT}
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        fill="currentColor"
        className="text-illustration-noise shrink-0"
        aria-hidden="true"
      >
        {STATES.map(({ w, rows }, state) => (
          <g
            key={state}
            className="noise-line-state"
            style={{ animationDelay: `${state * 0.2}s` }}
          >
            {rows.map((row, cell) => (
              <rect
                key={cell}
                x={cell * w}
                y={row * CELL_HEIGHT}
                width={w}
                height={CELL_HEIGHT}
              />
            ))}
          </g>
        ))}
      </svg>
    </div>
  );
}
