// TODO: fetch the supported excluded regions from the daemon via gRPC instead
// of hardcoding them here.
export const SupportedExcludedRegions = ['CN', 'RU'] as const;

export type SupportedExcludedRegion = (typeof SupportedExcludedRegions)[number];
