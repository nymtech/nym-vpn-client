const os = require('os');
const path = require('path');
const fs = require('fs');
const { spawn, spawnSync, execSync } = require('child_process');

let browserProcess;
let geckoDriverProcess;
let tauriDriver;
let tauriDevProcess;

const isCI = process.env.CI === 'true' || process.env.CI === true;
const isHeadless =
  process.env.HEADLESS === 'true' || process.env.HEADLESS === true;
const isDebug = process.env.DEBUG === 'true' || process.env.DEBUG === true;
const isWindows = process.platform === 'win32';
const isLinux = process.platform === 'linux';
const isMacOS = process.platform === 'darwin';

// Project root paths
const projectRootPath = path.resolve(__dirname, '..', '..');
const mainProjectPath = path.join(projectRootPath, 'nym-vpn-app');
const testProjectPath = path.join(
  projectRootPath,
  'nym-vpn-app',
  'tauri-webdriver-tests',
);

function createUniqueUserDataDir() {
  const tempDir = path.join(os.tmpdir(), `tauri-webdriver-${Date.now()}`);
  fs.mkdirSync(tempDir, { recursive: true });
  return tempDir;
}

const userDataDir = createUniqueUserDataDir();

const screenshotsDir = path.join(testProjectPath, 'screenshots');
if (!fs.existsSync(screenshotsDir)) {
  fs.mkdirSync(screenshotsDir, { recursive: true });
}

const reportsDir = path.join(testProjectPath, 'reports');
if (!fs.existsSync(reportsDir)) {
  fs.mkdirSync(reportsDir, { recursive: true });
}

function getGeckoDriverPath() {
  if (isMacOS) {
    try {
      const geckoPath = execSync('which geckodriver', {
        encoding: 'utf8',
      }).trim();
      if (geckoPath) return geckoPath;
    } catch (e) {
      if (fs.existsSync('/opt/homebrew/bin/geckodriver')) {
        return '/opt/homebrew/bin/geckodriver';
      } else if (fs.existsSync('/usr/local/bin/geckodriver')) {
        return '/usr/local/bin/geckodriver';
      }
    }

    throw new Error(
      'GeckoDriver not found. Please install using: brew install geckodriver',
    );
  }
  return 'geckodriver';
}

function startGeckoDriver() {
  return new Promise((resolve, reject) => {
    try {
      try {
        execSync('pkill -f geckodriver', { stdio: 'ignore' });
      } catch {}

      const geckoDriverPath = getGeckoDriverPath();
      console.log(`Starting geckodriver from: ${geckoDriverPath}`);

      geckoDriverProcess = spawn(
        geckoDriverPath,
        ['--port', '4444', '--log', 'trace'],
        {
          stdio: 'pipe',
          detached: false,
        },
      );

      geckoDriverProcess.stdout.on('data', (data) => {
        console.log(`GeckoDriver stdout: ${data}`);
      });

      geckoDriverProcess.stderr.on('data', (data) => {
        console.error(`GeckoDriver stderr: ${data}`);
      });

      geckoDriverProcess.on('exit', (code, signal) => {
        if (code !== 0) {
          console.error(
            `GeckoDriver exited with code ${code} and signal ${signal}`,
          );
        }
      });

      setTimeout(resolve, 3000);
    } catch (error) {
      console.error('Failed to start geckodriver:', error);
      reject(error);
    }
  });
}

function findTauriDriverPath() {
  try {
    if (isWindows) {
      return path.join(os.homedir(), '.cargo', 'bin', 'tauri-driver.exe');
    }
    return path.join(os.homedir(), '.cargo', 'bin', 'tauri-driver');
  } catch (error) {
    console.error('Failed to find Tauri driver path:', error);
    throw error;
  }
}

function findMsEdgeDriverPath() {
  const possibleLocations = [
    path.join(os.homedir(), 'Webdriver', 'msedgedriver.exe'),
    'C:\\Program Files\\Microsoft Edge WebDriver\\msedgedriver.exe',
    'C:\\Program Files (x86)\\Microsoft Edge WebDriver\\msedgedriver.exe',
    path.join(os.homedir(), 'Downloads', 'msedgedriver.exe'),
  ];

  for (const location of possibleLocations) {
    if (fs.existsSync(location)) {
      console.log(`Found msedgedriver at: ${location}`);
      return location;
    }
  }

  console.error('Microsoft Edge WebDriver not found.');
  return null;
}

function verifyFirefoxInstallation() {
  if (isMacOS) {
    if (!fs.existsSync('/Applications/Firefox.app')) {
      console.error('Firefox not found at /Applications/Firefox.app');
      console.error(
        'Please install Firefox or update the Firefox path in wdio.conf.js',
      );
    }
  }
}

verifyFirefoxInstallation();

exports.config = {
  runner: 'local',

  specs: [path.join(testProjectPath, 'src', 'tests', 'specs', '**', '*.js')],

  exclude: [...(isCI ? ['**/connection-*.js'] : [])],

  maxInstances: isCI ? 1 : 1,

  capabilities: isMacOS
    ? [
        {
          maxInstances: 1,
          browserName: 'firefox',
          'moz:firefoxOptions': {
            binary: '/Applications/Firefox.app/Contents/MacOS/firefox',
            args: [
              '--start-maximized',
              '--disable-extensions',
              ...(isHeadless ? ['--headless'] : []),
            ],
            prefs: {
              'security.sandbox.content.level': 0,
              'browser.cache.disk.enable': false,
              'browser.cache.memory.enable': false,
            },
          },
          acceptInsecureCerts: true,
          'webdriver:firefoxOptions': {
            binary: '/Applications/Firefox.app/Contents/MacOS/firefox',
          },
        },
      ]
    : [
        {
          maxInstances: 1,
          'tauri:options': {
            application: isWindows
              ? path.join(
                  mainProjectPath,
                  'src-tauri',
                  'target',
                  isDebug ? 'debug' : 'release',
                  isDebug ? 'nym-vpn-app.exe' : 'NymVPN.exe',
                )
              : path.join(
                  mainProjectPath,
                  'src-tauri',
                  'target',
                  isDebug ? 'debug' : 'release',
                  'nym-vpn-app',
                ),
            ...(isCI
              ? {
                  args: ['--ci-mode', '--mock-connections'],
                }
              : {}),
            ...(isWindows && {
              webviewArgs: [
                `--user-data-dir=${userDataDir}`,
                '--edge-skip-compat-layer-relaunch',
              ],
            }),
          },
        },
      ],

  // Connection settings
  hostname: 'localhost',
  port: 4444,

  // Logging and timeouts - increase timeouts for debug mode
  logLevel: 'info',
  bail: isCI ? 1 : 0,
  waitforTimeout: isCI ? 5000 : isDebug ? 20000 : 10000,
  connectionRetryTimeout: isDebug ? 180000 : 120000,
  connectionRetryCount: 3,

  // Framework and reporting
  framework: 'mocha',
  reporters: ['spec'],

  mochaOpts: {
    ui: 'bdd',
    timeout: isCI ? 30000 : isDebug ? 120000 : 60000,
  },

  // Hooks
  onPrepare: async function () {
    console.log(
      `Running in ${isCI ? 'CI' : 'local'} environment on ${process.platform} (${isDebug ? 'DEBUG' : 'RELEASE'} mode)`,
    );

    if (isMacOS && process.getuid && process.getuid() === 0) {
      console.error(
        'ERROR: Running as root user! This will cause problems with Firefox on macOS.',
      );
      console.error('Please run without sudo privileges.');
      process.exit(1);
    }

    // Clean up any existing processes on Linux
    if (isLinux) {
      console.log('Cleaning up any existing webdriver processes...');
      try {
        execSync('pkill -f "geckodriver"', { stdio: 'ignore' });
        execSync('pkill -f "tauri-driver"', { stdio: 'ignore' });
        execSync('pkill -f "webdriver"', { stdio: 'ignore' });
        // Kill processes on common WebDriver ports
        try {
          execSync('fuser -k 4444/tcp', { stdio: 'ignore' });
          execSync('fuser -k 4445/tcp', { stdio: 'ignore' });
        } catch (e) {}
      } catch (e) {
        // Ignore errors if processes don't exist
      }
    }

    // Check if node_modules exists in the main project path
    const nodeModulesPath = path.join(mainProjectPath, 'node_modules');
    if (!fs.existsSync(nodeModulesPath)) {
      console.log(
        'node_modules not found in the main project. Installing dependencies...',
      );
      try {
        console.log(`Running npm install in ${mainProjectPath}...`);
        const installResult = spawnSync('npm', ['install'], {
          stdio: 'inherit',
          cwd: mainProjectPath,
        });

        if (installResult.status !== 0) {
          console.error('Failed to install dependencies in the main project.');
          throw new Error('Dependency installation failed');
        }
        console.log('Dependencies installed successfully.');
      } catch (error) {
        console.error('Error installing dependencies:', error);
        throw error;
      }
    }

    if (isLinux) {
      if (isDebug) {
        console.log('Building Tauri application for Linux in DEBUG mode...');
        const buildResult = spawnSync('npm', ['run', 'app:dev'], {
          stdio: 'inherit',
          cwd: mainProjectPath,
        });

        if (buildResult.status !== 0) {
          throw new Error(
            'Failed to build Tauri application for Linux in DEBUG mode',
          );
        }
        console.log('Debug binary successfully built');
      } else {
        console.log('Building Tauri application for Linux in RELEASE mode...');
        const buildResult = spawnSync('npm', ['run', 'build:app'], {
          stdio: 'inherit',
          cwd: mainProjectPath,
        });

        if (buildResult.status !== 0) {
          throw new Error('Failed to build Tauri application for Linux');
        }
      }
    } else if (isMacOS) {
      console.log('Starting browser-based dev server...');
      browserProcess = spawn('npm', ['run', 'dev:browser'], {
        stdio: 'pipe',
        shell: true,
        cwd: mainProjectPath,
      });

      browserProcess.stdout.on('data', (data) => {
        const output = data.toString().trim();
        if (output) {
          console.log(`Vite stdout: ${output}`);
        }
      });

      browserProcess.stderr.on('data', (data) => {
        const error = data.toString().trim();
        if (error) {
          console.error(`Vite stderr: ${error}`);
        }
      });

      await startGeckoDriver();

      await new Promise((resolve, reject) => {
        const startTimeout = setTimeout(() => {
          console.log('Vite server is already running or timed out');
          resolve();
        }, 5000);

        browserProcess.stdout.on('data', (data) => {
          const output = data.toString();
          if (output.includes('Local:') || output.includes('ready in')) {
            clearTimeout(startTimeout);
            resolve();
          }
        });

        browserProcess.on('error', (err) => {
          clearTimeout(startTimeout);
          reject(err);
        });
      });
    } else if (isWindows) {
      const nodeModulesPath = path.join(mainProjectPath, 'node_modules');
      if (!fs.existsSync(nodeModulesPath)) {
        console.log('Installing dependencies for Tauri...');
        const installResult = spawnSync('npm', ['install'], {
          stdio: 'inherit',
          cwd: mainProjectPath,
        });

        if (installResult.status !== 0) {
          throw new Error('Failed to install dependencies for Tauri');
        }
      }

      const releaseBinaryPath = path.join(
        mainProjectPath,
        'src-tauri',
        'target',
        'release',
        'NymVPN.exe',
      );

      if (!fs.existsSync(releaseBinaryPath)) {
        console.log(
          'Building Tauri application for Windows in RELEASE mode...',
        );
        const buildResult = spawnSync('cargo', ['build', '--release'], {
          stdio: 'inherit',
          cwd: path.join(mainProjectPath, 'src-tauri'),
          shell: true,
        });

        if (buildResult.status !== 0) {
          throw new Error('Failed to build Tauri application for Windows');
        }
        console.log('Release binary successfully built');
      } else {
        console.log('Release binary already exists. Skipping build.');
      }
    }
  },

  beforeSession: async function () {
    if (!isMacOS) {
      const waitTime = isDebug ? 5000 : 2000;
      const tauriDriverPath = findTauriDriverPath();

      let args = [];

      if (isWindows) {
        const edgeDriverPath = findMsEdgeDriverPath();
        if (edgeDriverPath) {
          args = ['--native-driver', edgeDriverPath];
          console.log(`Using Edge native driver: ${edgeDriverPath}`);
        } else {
          console.warn(
            'Edge WebDriver not found, proceeding without --native-driver',
          );
        }
      }

      tauriDriver = spawn(tauriDriverPath, args, {
        stdio: [null, process.stdout, process.stderr],
        env: {
          ...process.env,
          RUST_LOG: isDebug ? 'debug' : 'info',
          ...(isDebug ? { TAURI_WEBVIEW_DRIVER: 'wry' } : {}),
        },
      });

      console.log(
        `Waiting ${waitTime / 1000} seconds for tauri-driver to initialize...`,
      );
      return new Promise((resolve) => setTimeout(resolve, waitTime));
    }
  },

  before: async function () {
    console.log('Setting up browser session...');

    if (isMacOS) {
      try {
        await browser.url('http://localhost:1420');
        console.log('Successfully navigated to application URL');

        try {
          const title = await browser.getTitle();
          console.log(`Page title: ${title}`);
        } catch (e) {
          console.warn('Could not get page title, but continuing...');
        }
      } catch (error) {
        console.error('Failed to navigate:', error);
        console.error('Continuing despite navigation error...');
      }
    } else {
      const waitTime = isDebug ? 15000 : 5000;
      console.log(
        `Waiting ${waitTime / 1000} seconds for app to be fully initialized...`,
      );
      await browser.pause(waitTime);

      console.log('App initialization waiting period complete.');
    }
  },

  afterTest: async function (test, context, { error }) {
    if (error) {
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const testName = test.title.replace(/\s+/g, '-').toLowerCase();
      const filepath = path.join(
        screenshotsDir,
        `${testName}-${timestamp}.png`,
      );

      try {
        await browser.saveScreenshot(filepath);
        console.log(`Screenshot saved to: ${filepath}`);
      } catch (screenshotError) {
        console.error(`Failed to save screenshot: ${screenshotError.message}`);
      }
    }
  },

  afterSession: function () {
    if (browserProcess) {
      try {
        browserProcess.kill();
      } catch (error) {
        console.error('Error killing browser process:', error);
      }
    }

    if (isMacOS && geckoDriverProcess) {
      try {
        geckoDriverProcess.kill();
        try {
          execSync('pkill -f geckodriver', { stdio: 'ignore' });
        } catch (e) {}

        try {
          const killScriptPath = path.join(
            testProjectPath,
            'scripts',
            'kill_processes.sh',
          );
          if (fs.existsSync(killScriptPath)) {
            console.log('Running kill_processes.sh as final cleanup step...');
            execSync(`bash ${killScriptPath}`, { stdio: 'inherit' });
          } else {
            console.warn('kill_processes.sh not found at:', killScriptPath);
          }
        } catch (e) {
          console.error('Error running kill_processes.sh:', e);
        }
      } catch (error) {
        console.error('Error killing geckodriver:', error);
      }
    }

    // Clean up tauri processes
    if (tauriDriver) {
      try {
        tauriDriver.kill();
      } catch (error) {
        console.error('Error killing tauri-driver:', error);
      }
    }
    if (tauriDevProcess) {
      try {
        console.log('Cleaning up Tauri dev process...');

        tauriDevProcess.kill();

        if (isLinux) {
          try {
            execSync('pkill -f "tauri dev"', { stdio: 'ignore' });
            execSync('pkill -f "cargo tauri"', { stdio: 'ignore' });
            execSync('pkill -f "npm run dev"', { stdio: 'ignore' });
            execSync('pkill -f "nym-vpn-app"', { stdio: 'ignore' });
            execSync('pkill -f "vite"', { stdio: 'ignore' });
          } catch (e) {
            console.log(
              'Some cleanup commands may have failed, but continuing...',
            );
          }
        } else if (isMacOS) {
          try {
            execSync('pkill -f "tauri dev"', { stdio: 'ignore' });
            execSync('pkill -f "nym-vpn-app"', { stdio: 'ignore' });
          } catch (e) {}
        } else if (isWindows) {
          try {
            execSync('taskkill /f /im "tauri.exe"', { stdio: 'ignore' });
            execSync('taskkill /f /im "nym-vpn-app.exe"', { stdio: 'ignore' });
            execSync('taskkill /f /im "cargo.exe"', { stdio: 'ignore' });
          } catch (e) {}
        }
      } catch (error) {
        console.error('Error during Tauri dev process cleanup:', error);
      }
    }

    // Final attempt to clean up any remaining processes
    console.log('Final process cleanup...');
    if (isLinux || isMacOS) {
      try {
        execSync('pkill -f "webdriver"', { stdio: 'ignore' });
        execSync('pkill -f "tauri"', { stdio: 'ignore' });
      } catch (e) {}
    }
  },
};
