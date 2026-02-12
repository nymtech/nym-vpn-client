export type NonNullableProps<T> = {
  [K in keyof T]: NonNullable<T[K]>;
};
