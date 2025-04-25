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

// Ensure directories exist
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
      // Try to get from PATH first
      const geckoPath = execSync('which geckodriver', {
        encoding: 'utf8',
      }).trim();
      if (geckoPath) return geckoPath;
    } catch (e) {
      // If not in PATH, try homebrew location
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
              '--disable-dev-shm-usage',
              '--no-sandbox',
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
            ...(!isDebug
              ? {
                  // For release mode, use the release build
                  application: isWindows
                    ? path.join(
                        mainProjectPath,
                        'src-tauri',
                        'target',
                        'release',
                        'nym-vpn-app.exe',
                      )
                    : path.join(
                        mainProjectPath,
                        'src-tauri',
                        'target',
                        'release',
                        'nym-vpn-app',
                      ),
                }
              : {
                  application: isWindows
                    ? path.join(
                        mainProjectPath,
                        'src-tauri',
                        'target',
                        'debug',
                        'nym-vpn-app.exe',
                      )
                    : path.join(
                        mainProjectPath,
                        'src-tauri',
                        'target',
                        'debug',
                        'nym-vpn-app',
                      ),
                }),
            ...(isCI
              ? {
                  args: ['--ci-mode', '--mock-connections'],
                }
              : {}),
          },
        },
      ],

  // Connection settings
  hostname: 'localhost',
  port: 4444,

  // Logging and timeouts
  logLevel: 'info',
  bail: isCI ? 1 : 0,
  waitforTimeout: isCI ? 5000 : 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // Framework and reporting
  framework: 'mocha',
  reporters: ['spec'],

  mochaOpts: {
    ui: 'bdd',
    timeout: isCI ? 30000 : 60000,
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
        console.log('Starting Tauri development server for Linux...');
        tauriDevProcess = spawn('npm', ['run', 'tauri', 'dev'], {
          stdio: 'inherit',
          env: {
            ...process.env,
            RUST_LOG: 'info,nym_vpn_app=trace',
            RUSTFLAGS: '-C link-args=-Wl,-rpath,/usr/lib/x86_64-linux-gnu',
          },
          cwd: mainProjectPath,
          detached: true,
        });

        console.log('Waiting for Tauri dev server to start...');
        await new Promise((resolve) => setTimeout(resolve, 15000));
      } else {
        console.log('Building Tauri application for Linux in RELEASE mode...');
        const buildEnv = {
          ...process.env,
          RUST_LOG: 'info,nym_vpn_app=trace',
          RUSTFLAGS: '-C link-args=-Wl,-rpath,/usr/lib/x86_64-linux-gnu',
        };

        const buildResult = spawnSync('npm', ['run', 'tauri', 'build'], {
          stdio: 'inherit',
          env: buildEnv,
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
    } else {
      // For Windows platforms
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

      if (isDebug) {
        console.log('Starting Tauri development server for Windows...');
        tauriDevProcess = spawn('npm', ['run', 'tauri', 'dev'], {
          stdio: 'inherit',
          env: {
            ...process.env,
            RUST_LOG: 'info,nym_vpn_app=trace',
          },
          cwd: mainProjectPath,
          detached: true,
        });

        console.log('Waiting for Tauri dev server to start...');
        await new Promise((resolve) => setTimeout(resolve, 15000));
      } else {
        console.log(
          'Building Tauri application for Windows in RELEASE mode...',
        );
        const buildResult = spawnSync('npm', ['run', 'tauri', 'build'], {
          stdio: 'inherit',
          env: {
            ...process.env,
            RUST_LOG: 'info,nym_vpn_app=trace',
          },
          cwd: mainProjectPath,
        });

        if (buildResult.status !== 0) {
          throw new Error('Failed to build Tauri application in release mode');
        }
      }
    }
  },

  beforeSession: async function () {
    if (!isMacOS) {
      if (!isDebug) {
        // In release mode, we need to start tauri-driver to control the app
        console.log('Starting tauri-driver for release build...');
        const tauriDriverPath = findTauriDriverPath();

        tauriDriver = spawn(tauriDriverPath, [], {
          stdio: [null, process.stdout, process.stderr],
          env: {
            ...process.env,
            RUST_LOG: 'info',
          },
        });
      } else {
        console.log('Starting tauri-driver for dev server...');
        const tauriDriverPath = findTauriDriverPath();

        tauriDriver = spawn(tauriDriverPath, [], {
          stdio: [null, process.stdout, process.stderr],
          env: {
            ...process.env,
            RUST_LOG: 'info',
          },
        });
      }

      return new Promise((resolve) => setTimeout(resolve, 2000));
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
    // Clean up processes
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
        tauriDevProcess.kill();
        if (isLinux || isMacOS) {
          try {
            execSync('pkill -f "tauri dev"', { stdio: 'ignore' });
          } catch (e) {}
        } else if (isWindows) {
          try {
            execSync('taskkill /f /im "tauri.exe"', { stdio: 'ignore' });
          } catch (e) {}
        }
      } catch (error) {
        console.error('Error killing tauri dev process:', error);
      }
    }
  },
};
