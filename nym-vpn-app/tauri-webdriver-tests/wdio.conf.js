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
              : {}),
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
        console.log('Waiting for Tauri dev server to start...');

        tauriDevProcess = spawn('npm', ['run', 'tauri', 'dev'], {
          stdio: 'inherit',
          env: {
            ...process.env,
            RUST_LOG: 'info,nym_vpn_app=trace',
            RUSTFLAGS: '-C link-args=-Wl,-rpath,/usr/lib/x86_64-linux-gnu',
          },
          cwd: mainProjectPath,
          detached: true,
          shell: true,
        });

        let isReady = false;
        const maxWaitTime = 60000;
        const startTime = Date.now();

        // We'll wait here for the app to be ready
        console.log(
          'Waiting for Tauri dev server to initialize (up to 60 seconds)...',
        );
        await new Promise((resolve) => {
          const checkInterval = setInterval(() => {
            // Check if we've exceeded maximum wait time
            if (Date.now() - startTime > maxWaitTime) {
              clearInterval(checkInterval);
              console.log(
                'Timed out waiting for Tauri dev server, continuing anyway...',
              );
              resolve();
            }

            if (isReady) {
              clearInterval(checkInterval);
              resolve();
            }
          }, 1000);

          if (tauriDevProcess.stdout) {
            tauriDevProcess.stdout.on('data', (data) => {
              const output = data.toString();
              if (
                output.includes('Finished `dev`') ||
                output.includes('Running DevCommand') ||
                output.includes('Starting webview window')
              ) {
                console.log('Detected Tauri dev server is ready');
                isReady = true;
              }
            });
          }

          setTimeout(() => {
            if (!isReady) {
              console.log('Minimum wait time reached, continuing...');
              isReady = true;
            }
          }, 25000);
        });

        console.log('Dev server detected, waiting for app initialization...');
        await new Promise((resolve) => setTimeout(resolve, 15000));
      } else {
        // Original release build code
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
          shell: true,
        });

        console.log('Waiting for Tauri dev server to start...');
        let isReady = false;
        const maxWaitTime = 60000;
        const startTime = Date.now();

        await new Promise((resolve) => {
          const checkInterval = setInterval(() => {
            if (Date.now() - startTime > maxWaitTime) {
              clearInterval(checkInterval);
              console.log(
                'Timed out waiting for Tauri dev server, continuing anyway...',
              );
              resolve();
            }

            if (isReady) {
              clearInterval(checkInterval);
              resolve();
            }
          }, 1000);

          if (tauriDevProcess.stdout) {
            tauriDevProcess.stdout.on('data', (data) => {
              const output = data.toString();
              if (
                output.includes('Finished `dev`') ||
                output.includes('Running DevCommand') ||
                output.includes('Starting webview window')
              ) {
                console.log('Detected Tauri dev server is ready');
                isReady = true;
              }
            });
          }

          setTimeout(() => {
            if (!isReady) {
              console.log('Minimum wait time reached, continuing...');
              isReady = true;
            }
          }, 25000);
        });

        await new Promise((resolve) => setTimeout(resolve, 10000));
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
      // For debug mode, we need a longer wait since the app takes longer to compile and start
      const waitTime = isDebug ? 5000 : 2000;

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

    // Enhanced cleanup for tauri dev process
    if (tauriDevProcess) {
      try {
        console.log('Cleaning up Tauri dev process...');

        // Attempt to kill the process
        tauriDevProcess.kill();

        // Platform-specific additional cleanup
        if (isLinux) {
          try {
            // Try multiple commands to ensure cleanup
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
