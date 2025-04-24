const isCI = process.env.CI === 'true' || process.env.CI === true;

const config = {
  connection: {
    enabled: !isCI,
    timeouts: {
      connection: isCI ? 5000 : 15000,
      disconnection: isCI ? 3000 : 5000,
    },
  },

  location: {
    enabled: true,
    useMocks: isCI,
  },

  ui: {
    enabled: true,
  },
};

function isTestEnabled(testType) {
  return config[testType]?.enabled ?? false;
}

function getTimeout(testType, timeoutType) {
  return config[testType]?.timeouts?.[timeoutType] ?? 10000; // Default 10s
}

function shouldUseMocks(testType) {
  return config[testType]?.useMocks ?? false;
}

function conditionalTest(testType) {
  return isTestEnabled(testType) ? it : it.skip;
}

function conditionalDescribe(testType) {
  return isTestEnabled(testType) ? describe : describe.skip;
}

module.exports = {
  isCI,
  isTestEnabled,
  getTimeout,
  shouldUseMocks,
  conditionalTest,
  conditionalDescribe,
};
