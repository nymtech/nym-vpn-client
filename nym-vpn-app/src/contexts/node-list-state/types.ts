export type Focused = {
  type: 'gateway' | 'region' | 'country';
  // country 2-letter code | region name + country 2-letter code | gateway ID
  key: string;
};
