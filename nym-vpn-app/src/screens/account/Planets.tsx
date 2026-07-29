import { motion, useReducedMotion } from 'motion/react';

/**
 * Decorative background glows, matching the `Planet` layers in the plan-selection
 * design. Each one is a blurred radial gradient rather than an exported asset:
 * Figma draws them as a blurred ellipse filled with a `#0F6E42 → #0A0A0A`
 * gradient, which is a brand-green glow fading into the page background — that
 * reproduces exactly in CSS, costs no bytes, and follows the theme instead of
 * baking in the dark palette.
 *
 * `size` is a percentage of the layer's width and `cx`/`cy` are the glow's centre,
 * so the composition holds at any window size — the app runs anywhere from 250px
 * to 800px wide.
 */
const PLANETS = [
  {
    // Largest, centre just off the left edge, behind the heading.
    size: 65,
    cx: 3,
    cy: 43,
    drift: { x: [0, 18, -8, 0], y: [0, -22, 12, 0] },
    duration: 23,
  },
  {
    // Off the right edge, upper.
    size: 49,
    cx: 89,
    cy: 30,
    drift: { x: [0, -12, 9, 0], y: [0, 16, -10, 0] },
    duration: 19,
  },
  {
    // Off the right edge, lower. Set further apart than the design, which has the
    // two right-hand glows overlapping — they read as one blob at that spacing.
    size: 42,
    cx: 91,
    cy: 56,
    drift: { x: [0, 10, -14, 0], y: [0, -12, 16, 0] },
    duration: 29,
  },
  // Not `as const`: motion's keyframe types require mutable arrays.
];

export function Planets() {
  // A perpetual ambient animation is exactly the case this preference exists for.
  const reduceMotion = useReducedMotion();

  return (
    // `-inset-4` cancels MainLayout's p-4 so the glows reach the window edges,
    // landing exactly on its padding box so they add no scrollable overflow.
    // `overflow-hidden` clips the parts that sit outside.
    <div
      aria-hidden
      className="pointer-events-none absolute -inset-4 overflow-hidden"
    >
      {PLANETS.map((planet, index) => (
        <motion.div
          key={index}
          // opacity/blur/stop calibrated against the design render, measured at the
          // left and right edge columns: this lands peak "greenness" 30/23 against
          // Figma's 28/24. Centring the gradient (below) costs peak intensity, so
          // this is higher than it looks like it should be.
          className="absolute rounded-full opacity-60 blur-[28px]"
          style={{
            width: `${planet.size}%`,
            aspectRatio: '1',
            left: `${planet.cx}%`,
            top: `${planet.cy}%`,
            // Percentage margins resolve against the containing block's *width* in
            // both axes, and the box is square, so these offset it by exactly half
            // its size — centring it on (cx, cy) whatever the size.
            marginLeft: `${-planet.size / 2}%`,
            marginTop: `${-planet.size / 2}%`,
            // Centred, with `closest-side` so the radius is half the box and the
            // fade completes at 78% of that — comfortably inside the element.
            // An off-centre gradient (the design's is directional) is still part
            // way opaque when it meets the box edge, and the box truncates it into
            // a visible straight cut that blur only softens.
            background:
              'radial-gradient(circle closest-side at 50% 50%, var(--color-brand-primary), transparent 78%)',
          }}
          // Only `x`/`y` are animated, so this stays on the compositor and the
          // expensive blur is rasterized once rather than every frame.
          animate={
            reduceMotion ? undefined : { x: planet.drift.x, y: planet.drift.y }
          }
          transition={{
            duration: planet.duration,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
        />
      ))}
    </div>
  );
}
