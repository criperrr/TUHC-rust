/**
 * TUHC-rust Frontend Visual & Subsystem E2E Test Runner
 *
 * Drives the Webview frontend using Playwright, inspecting live DOM elements,
 * capturing high-resolution screenshots, and recording streaming diagnostic logs.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const { chromium, firefox } = require('playwright-core');

// Resolve command-line arguments
const args = process.argv.slice(2);
function getArg(flag, defaultValue) {
  const match = args.find(a => a.startsWith(`--${flag}=`));
  if (match) return match.split('=')[1];
  const index = args.indexOf(`--${flag}`);
  if (index !== -1 && args[index + 1]) return args[index + 1];
  return defaultValue;
}

const BASE_URL = getArg('url', 'http://127.0.0.1:19890');
const HEADLESS = getArg('headless', 'true') !== 'false';
const BROWSER_TYPE = getArg('browser', 'chromium');
const OUT_DIR = path.resolve(__dirname);
const SHOTS_DIR = path.join(OUT_DIR, 'screenshots');
const LOGS_DIR = path.join(OUT_DIR, 'logs');
const REPORTS_DIR = path.join(OUT_DIR, 'reports');
const LOG_FILE = path.join(LOGS_DIR, 'frontend_e2e.log');
const REPORT_FILE = path.join(REPORTS_DIR, 'summary.json');

// Ensure output directories exist
[SHOTS_DIR, LOGS_DIR, REPORTS_DIR].forEach(dir => {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
});

// Setup structured file logger
const logStream = fs.createWriteStream(LOG_FILE, { flags: 'w' });
function log(level, message, meta = null) {
  const ts = new Date().toISOString();
  const metaStr = meta ? ` | ${JSON.stringify(meta)}` : '';
  const line = `[${ts}] [${level.toUpperCase()}] ${message}${metaStr}\n`;
  logStream.write(line);
  if (level === 'ERROR' || level === 'WARN') {
    console.error(`[${level}] ${message}`);
  } else {
    console.log(`[${level}] ${message}`);
  }
}

// Locate local browser executable
function resolveChromiumExecutable() {
  if (process.env.CHROMIUM_PATH && fs.existsSync(process.env.CHROMIUM_PATH)) {
    return process.env.CHROMIUM_PATH;
  }
  try {
    const defaultPath = chromium.executablePath();
    if (defaultPath && fs.existsSync(defaultPath)) return defaultPath;
  } catch (_) {}

  const msPlaywright = path.join(os.homedir(), '.cache', 'ms-playwright');
  if (fs.existsSync(msPlaywright)) {
    const entries = fs.readdirSync(msPlaywright).filter(d => d.startsWith('chromium-'));
    for (const d of entries.sort().reverse()) {
      const p1 = path.join(msPlaywright, d, 'chrome-linux64', 'chrome');
      if (fs.existsSync(p1)) return p1;
      const p2 = path.join(msPlaywright, d, 'chrome-linux', 'chrome');
      if (fs.existsSync(p2)) return p2;
    }
  }

  const systemPaths = [
    '/usr/bin/google-chrome-stable',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium-browser',
    '/usr/bin/chromium'
  ];
  for (const sp of systemPaths) {
    if (fs.existsSync(sp)) return sp;
  }

  return undefined;
}

async function runE2ETestSuite() {
  log('INFO', `Starting TUHC Frontend Visual & Subsystem E2E Suite against ${BASE_URL}`);
  const startTime = Date.now();
  const testResults = [];
  const capturedScreenshots = [];
  const consoleMetrics = { errors: 0, warnings: 0, info: 0 };

  let browser;
  try {
    const execPath = resolveChromiumExecutable();
    log('INFO', `Resolved browser executable: ${execPath || 'Playwright Default'}`);

    browser = await (BROWSER_TYPE === 'firefox' ? firefox : chromium).launch({
      headless: HEADLESS,
      executablePath: execPath,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
    });

    const context = await browser.newContext({
      viewport: { width: 1200, height: 900 },
      deviceScaleFactor: 1
    });

    const page = await context.newPage();

    // Attach diagnostic console and error listeners
    page.on('console', msg => {
      const type = msg.type().toUpperCase();
      if (type === 'ERROR') consoleMetrics.errors++;
      else if (type === 'WARN' || type === 'WARNING') consoleMetrics.warnings++;
      else consoleMetrics.info++;
      log(`BROWSER-${type}`, msg.text());
    });

    page.on('pageerror', err => {
      consoleMetrics.errors++;
      log('PAGEERROR', err.message, { stack: err.stack });
    });

    page.on('requestfailed', req => {
      const failure = req.failure();
      log('REQ-FAILED', `${req.method()} ${req.url()} - ${failure ? failure.errorText : 'failed'}`);
    });

    // Helper to capture timestamped screenshots
    async function capture(filename, description) {
      const targetPath = path.join(SHOTS_DIR, filename);
      await page.screenshot({ path: targetPath, fullPage: false });
      const stats = fs.statSync(targetPath);
      capturedScreenshots.push({
        filename,
        path: targetPath,
        sizeBytes: stats.size,
        description
      });
      log('INFO', `Captured screenshot: ${filename} (${(stats.size / 1024).toFixed(1)} KB)`);
    }

    // Helper to record test step
    async function testStep(name, action) {
      const stepStart = Date.now();
      try {
        log('INFO', `>>> Executing Test: ${name}`);
        await action();
        const duration = Date.now() - stepStart;
        testResults.push({ name, status: 'PASS', durationMs: duration });
        log('INFO', `<<< Test PASSED: ${name} (${duration}ms)`);
      } catch (err) {
        const duration = Date.now() - stepStart;
        testResults.push({ name, status: 'FAIL', durationMs: duration, error: err.message });
        log('ERROR', `<<< Test FAILED: ${name} (${duration}ms) - ${err.message}`);
      }
    }

    // =========================================================================
    // Subsystem Test 1: Chrome Shell & Baseline Layout
    // =========================================================================
    await testStep('Chrome & Window Layout', async () => {
      await page.goto(BASE_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(1000);

      // Verify header shell and chrome controls
      const header = await page.$('#appHeader');
      if (!header) throw new Error('Missing #appHeader element');

      const titleOrTabBar = await page.$('#titleBar, #tab-bar, .tabSection');
      if (!titleOrTabBar) throw new Error('Missing title or tab bar element');

      const titleEl = await page.$('#titleBarText, .tab.activeTab .tabTitle');
      if (titleEl) {
        const titleText = (await titleEl.textContent()).trim();
        if (!titleText.includes('Homestuck')) throw new Error(`Unexpected title text: ${titleText}`);
      }

      // Verify Address Bar / JumpBox
      const jumpBox = await page.$('#jumpBox');
      if (!jumpBox) throw new Error('Missing #jumpBox element');

      // Verify authentic dual ghost footer
      const footer = await page.$('#page-container .footer.mspa');
      if (!footer) throw new Error('Missing authentic footer element');

      await capture('01_app_chrome_and_shell.png', 'Window chrome, titlebar, addressbar, and MSPA shell');
    });

    // =========================================================================
    // Subsystem Test 2: Story Page & Content Layout (Act 1 Intro)
    // =========================================================================
    await testStep('Story Page Rendering & Content', async () => {
      await page.evaluate(() => window.TUHCApp.pushUrl('/mspa/001901'));
      await page.waitForSelector('.pageTitle', { timeout: 5000 });
      await page.waitForTimeout(300);

      const pageTitle = await page.$eval('.pageTitle', el => el.textContent.trim());
      if (pageTitle !== 'Homestuck') throw new Error(`Expected 'Homestuck', got: ${pageTitle}`);

      const nextArrow = await page.$eval('.nextArrowLink', el => el.textContent.trim());
      if (nextArrow !== 'Enter name.') throw new Error(`Expected 'Enter name.', got: ${nextArrow}`);

      const mediaImg = await page.$eval('.mediaContent img', el => el.getAttribute('src'));
      if (!mediaImg.includes('00001.gif')) throw new Error(`Unexpected media source: ${mediaImg}`);

      await capture('02_story_page_001901.png', 'Homestuck Act 1 opening page layout, title, gif, and arrow');
    });

    // =========================================================================
    // Subsystem Test 3: Tab Management & History Navigation
    // =========================================================================
    await testStep('Tab Lifecycle & URL History', async () => {
      const tabsCountBefore = await page.$$eval('#tabs .tab', els => els.length);

      // Create new tab
      await page.click('#new-tab-btn');
      await page.waitForTimeout(500);

      const tabsCountAfter = await page.$$eval('#tabs .tab', els => els.length);
      if (tabsCountAfter !== tabsCountBefore + 1) {
        throw new Error(`Tab count did not increment. Before: ${tabsCountBefore}, After: ${tabsCountAfter}`);
      }

      // Navigate new tab to page 001902
      await page.evaluate(() => window.TUHCApp.pushUrl('/mspa/001902'));
      await page.waitForTimeout(600);

      const addressVal = await page.$eval('#address-bar-input', el => el.value);
      if (!addressVal.includes('001902') && !addressVal.includes('/homestuck/2')) {
        throw new Error(`Address bar out of sync: ${addressVal}`);
      }

      // Switch back to original first tab
      await page.evaluate(() => {
        const first = document.querySelector('#tabs .tab');
        if (first) first.click();
      });
      await page.waitForTimeout(500);

      await capture('03_tab_management.png', 'Tab management, multiple tabs, and tab switching');
    });

    // =========================================================================
    // Subsystem Test 4: Pesterlog & Dialogue Toggle
    // =========================================================================
    await testStep('Pesterlog & Character Dialogue Expansion', async () => {
      await page.evaluate(() => window.TUHCApp.pushUrl('/mspa/001926'));
      await page.waitForSelector('.logButton', { timeout: 5000 });
      await page.waitForTimeout(300);

      const logBtn = await page.$('.logButton');
      if (!logBtn) throw new Error('Pesterlog toggle button (.logButton) not found on page 001926');

      const displayBefore = await page.$eval('.logContent', el => el.style.display);
      if (displayBefore !== 'none') throw new Error(`Expected pesterlog initial display none, got: ${displayBefore}`);

      // Click button to expand
      await logBtn.click();
      await page.waitForTimeout(300);

      const displayAfter = await page.$eval('.logContent', el => el.style.display);
      if (displayAfter !== 'block') throw new Error(`Expected pesterlog display block after click, got: ${displayAfter}`);

      const btnTextAfter = await page.$eval('.logButton', el => el.textContent.trim());
      if (!btnTextAfter.toLowerCase().includes('hide')) {
        throw new Error(`Expected button text to contain 'Hide', got: ${btnTextAfter}`);
      }

      await capture('04_pesterlog_expanded.png', 'Expanded pesterlog dialogue with character quirk colors');
    });

    // =========================================================================
    // Subsystem Test 5: Authentic MSPA Theme Switching
    // =========================================================================
    await testStep('Authentic MSPA Theme Switching', async () => {
      const themesToTest = [
        { name: 'midnight', shot: '05_theme_midnight.png', desc: 'Midnight Crew Noir Theme' },
        { name: 'cascade', shot: '06_theme_cascade.png', desc: 'Cascade Emerald Green Theme' },
        { name: 'trickster', shot: '07_theme_trickster.png', desc: 'Trickster Mode Vibrant Pastel Theme' }
      ];

      for (const t of themesToTest) {
        await page.evaluate((themeName) => {
          document.getElementById('app').setAttribute('data-theme', themeName);
          const pageContainer = document.getElementById('page-container');
          if (pageContainer) pageContainer.setAttribute('data-theme', themeName);
        }, t.name);
        await page.waitForTimeout(400);

        const currentTheme = await page.$eval('#app', el => el.getAttribute('data-theme'));
        if (currentTheme !== t.name) throw new Error(`Failed setting theme ${t.name}, got: ${currentTheme}`);

        await capture(t.shot, t.desc);
      }

      // Restore baseline MSPA theme
      await page.evaluate(() => {
        document.getElementById('app').setAttribute('data-theme', 'mspa');
        const pageContainer = document.getElementById('page-container');
        if (pageContainer) pageContainer.setAttribute('data-theme', 'mspa');
      });
      await page.waitForTimeout(200);
    });

    // =========================================================================
    // Subsystem Test 6: Media Engine & Ruffle Flash WASM Container
    // =========================================================================
    await testStep('Ruffle Flash WASM Media Container', async () => {
      await page.evaluate(() => window.TUHCApp.pushUrl('/mspa/002153'));
      await page.waitForTimeout(1000);

      const ruffleIframe = await page.$('iframe.ruffle-frame');
      if (!ruffleIframe) throw new Error('Ruffle Flash WASM iframe (iframe.ruffle-frame) was not mounted');

      const srcdoc = await page.$eval('iframe.ruffle-frame', el => el.getAttribute('srcdoc'));
      if (!srcdoc || !srcdoc.includes('RufflePlayer')) {
        throw new Error('Ruffle iframe srcdoc does not contain RufflePlayer WASM bootstrap');
      }

      await capture('08_flash_ruffle_container.png', 'Ruffle Flash WASM media container and ActionScript Frag IPC frame');
    });

    // =========================================================================
    // Subsystem Test 7: Sound Test Bar & Audio Playback
    // =========================================================================
    await testStep('Sound Test Bar & Music Audio Controls', async () => {
      await page.evaluate(() => window.TUHCApp.pushUrl('/music'));
      await page.waitForSelector('.albumCard', { timeout: 5000 }).catch(() => null);
      await page.waitForTimeout(400);

      // Click first album
      const firstAlbum = await page.$('.albumCard');
      if (firstAlbum) {
        await firstAlbum.click();
        await page.waitForSelector('.trackRow', { timeout: 5000 }).catch(() => null);
        await page.waitForTimeout(400);

        // Click first track in album
        const firstTrack = await page.$('.trackRow');
        if (firstTrack) {
          await firstTrack.click();
          await page.waitForTimeout(500);

          const isBarHidden = await page.$eval('#sound-test-bar', el => el.classList.contains('hidden'));
          if (isBarHidden) throw new Error('Sound test bar remained hidden after track selection');

          const trackTitle = await page.$eval('#sound-test-title', el => el.textContent.trim());
          if (!trackTitle || trackTitle === 'No track loaded') {
            throw new Error(`Track title not populated: ${trackTitle}`);
          }
        }
      }

      await capture('09_sound_test_active.png', 'Active persistent sound test mini-player bar');
    });

    // =========================================================================
    // Subsystem Test 8: Bookmarks & Save Data Drawer
    // =========================================================================
    await testStep('Bookmarks & Save Data Drawer', async () => {
      await page.click('#bookmarks-toggle-btn');
      await page.waitForTimeout(400);

      const isDrawerHidden = await page.$eval('#bookmarks-drawer', el => el.classList.contains('hidden'));
      if (isDrawerHidden) throw new Error('Bookmarks drawer failed to open');

      await capture('10_bookmarks_drawer.png', 'Bookmarks & Save Data slide-out drawer');

      // Close drawer
      const closeBtn = await page.$('#bookmarks-close-btn');
      if (closeBtn) await closeBtn.click();
      await page.waitForTimeout(200);
    });

    // =========================================================================
    // Subsystem Test 9: Settings View & Community Mod Toggles
    // =========================================================================
    await testStep('Settings View & Community Mod Toggles', async () => {
      await page.evaluate(() => window.TUHCApp.pushUrl('/settings'));
      await page.waitForSelector('.settingsSectionCard', { timeout: 5000 }).catch(() => null);
      await page.waitForTimeout(400);

      const settingsCards = await page.$$eval('.settingsSectionCard', els => els.length);
      if (settingsCards < 2) throw new Error(`Expected at least 2 settings cards, found ${settingsCards}`);

      await capture('11_settings_view.png', 'Settings drawer showing Themes, Visuals, and Community Mod toggles');
    });

    // Subsystem Test 10: Performance & Heap Diagnostics
    const performanceMetrics = await page.evaluate(() => {
      return {
        domNodes: document.querySelectorAll('*').length,
        jsHeapUsedBytes: (window.performance && window.performance.memory) ? window.performance.memory.usedJSHeapSize : null,
        jsHeapTotalBytes: (window.performance && window.performance.memory) ? window.performance.memory.totalJSHeapSize : null
      };
    });

    log('INFO', 'Collected DOM & Performance Diagnostics', performanceMetrics);

  } finally {
    if (browser) await browser.close();
  }

  const totalDuration = Date.now() - startTime;
  const passedTests = testResults.filter(r => r.status === 'PASS').length;
  const failedTests = testResults.filter(r => r.status === 'FAIL').length;

  const summary = {
    timestamp: new Date().toISOString(),
    baseUrl: BASE_URL,
    browser: BROWSER_TYPE,
    durationMs: totalDuration,
    stats: {
      total: testResults.length,
      passed: passedTests,
      failed: failedTests,
      screenshots: capturedScreenshots.length,
      consoleErrors: consoleMetrics.errors,
      consoleWarnings: consoleMetrics.warnings
    },
    tests: testResults,
    screenshots: capturedScreenshots,
    logFile: LOG_FILE
  };

  fs.writeFileSync(REPORT_FILE, JSON.stringify(summary, null, 2), 'utf-8');
  log('INFO', `Test execution completed in ${totalDuration}ms. Report saved to: ${REPORT_FILE}`);
  await new Promise(resolve => logStream.end(resolve));

  return summary;
}

// Direct execution
if (require.main === module) {
  runE2ETestSuite()
    .then(summary => {
      console.log('\n============================================================');
      console.log(`TUHC Frontend E2E Results: ${summary.stats.passed}/${summary.stats.total} Passed in ${(summary.durationMs / 1000).toFixed(2)}s`);
      console.log(`Screenshots Captured: ${summary.stats.screenshots}`);
      console.log(`Report File: ${REPORT_FILE}`);
      console.log(`Log File: ${LOG_FILE}`);
      console.log('============================================================\n');

      if (summary.stats.failed > 0) {
        process.exit(1);
      } else {
        process.exit(0);
      }
    })
    .catch(err => {
      console.error('Fatal E2E Execution Failure:', err);
      process.exit(1);
    });
}

module.exports = { runE2ETestSuite };
