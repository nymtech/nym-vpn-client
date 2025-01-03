import type { Config } from 'tailwindcss';
import defaultTheme from 'tailwindcss/defaultTheme';

export default {
  content: ['./index.html', './src/error.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    colors: {
      transparent: 'transparent',
      current: 'currentColor',
      'baltic-sea': {
        // [D] bg for network modes
        jaguar: '#2B2831',
        // [L] status-line title text + icon
        // [L] connection timer text
        // [L] "Connecting" status text
        // [L] main titles text
        // [L] network mode title text + icon
        // [L] node location select text value + icon + label
        // [D] button text
        DEFAULT: '#1C1B1F',
      },
      // [DL] secondary accent
      cornflower: '#7075FF',
      // [DL] error status text
      teaberry: '#E33B5A',
      comet: '#625B71',
      // [DL] Daemon dot 'Ok'
      'vert-menthe': '#2BC761',
      // [DL] Daemon dot 'NonCompat'
      'liquid-lava': '#F97316',
      // [DL] "Connected" status bg (combined with 10% opacity)
      'vert-prasin': '#47C45D',
      // [D] main titles text
      // [D] connection timer text
      // [D] "Connecting" status text
      // [L] bg for top-bar nav
      // [L] bg for network modes
      // [L] button text
      white: '#FFF',
      'flawed-white': '#FFFBFE',
      black: '#000',
      mercury: {
        // [D] status-line title text + icon
        // [D] network mode title text + icon
        // [D] node location select text value + icon + label
        pinkish: '#E6E1E5',
        DEFAULT: '#E1EFE7',
        // [D] network mode desc text
        // [D] "Connection time"
        // [D] main status desc text
        mist: '#938F99',
      },
      // [DL] "Disconnected" status text
      'coal-mine': { dark: '#56545A', light: '#A4A4A4' },
      // [L] "Connection time"
      // [L] main status desc text
      'dim-gray': '#696571',
      // [L] network mode desc text
      // [L] node location select outline
      // [L] connection status bg (combined with 10% opacity)
      'cement-feet': '#79747E',
      // [D] node location select outline
      'gun-powder': '#49454F',
      // [D] top-bar icon
      'laughing-jack': '#CAC4D0',
      // [L] button bg in disabled state
      'wind-chime': '#DEDEE1',
      // [D] connection status bg (combined with 15% opacity)
      oil: '#313033',
      // [L] login screen color of nym logo
      ghost: '#C7C7D1',
      // [D] button bg 'Stop' state
      'dusty-grey': '#CECCD1',
      // [D] radio-group bg hover
      onyx: '#3A373F',
      // [L] radio-group bg hover
      platinum: '#E2E8EC',
      // [L] input border ring hover
      aluminium: '#8DA3B1',
      // [D] bg for snackbar
      'poivre-noir': '#484649',
      // [L] bg for snackbar
      seashell: '#FFF2EF',

      /// NEON-SKIN UPDATE //////////////////////////////
      // [D] main bg
      // [L] NYM logo fg
      ash: '#242B2D',
      octave: {
        // [D] surface bg
        DEFAULT: '#32373D',
        // [D] top-bar bg
        arsenic: '#374042',
      },
      // [L] main bg
      'faded-lavender': '#EBEEF4',
      // [DL] Main accent
      malachite: {
        DEFAULT: '#14E76F',
        // [L] link text, button ring
        moss: '#0B8A42',
      },
    },
    extend: {
      fontFamily: {
        sans: ['Lato', ...defaultTheme.fontFamily.sans],
        icon: [
          'Material Symbols Outlined',
          {
            fontVariationSettings: '"opsz" 24;',
          },
        ],
      },
    },
  },
  plugins: [],
  // Toggling dark mode manually
  darkMode: 'class',
} satisfies Config;
