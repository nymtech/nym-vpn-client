/** @type {import("prettier").Config} */

const config = {
  tabWidth: 2,
  semi: true,
  singleQuote: true,
  // this is the default be for the sake of clarity, be explicit
  endOfLine: 'lf',
  plugins: ['prettier-plugin-tailwindcss'],
};

export default config;
