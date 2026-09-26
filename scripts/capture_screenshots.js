/**
 * BeReal Studio — High-Fidelity Automated Screenshot Capture Engine
 *
 * Uses puppeteer-core connected to the local Edge/Chrome browser to accurately
 * hydrate demo data, wait for Svelte reactivity & CSS transitions, and capture
 * pixel-perfect screenshots for documentation.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';
import puppeteer from 'puppeteer-core';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');
const SCREENSHOT_DIR = path.resolve(ROOT_DIR, 'docs', 'screenshots');

const VIEWS = [
  { name: '01_home_dashboard.png', view: 'home', label: 'Home Dashboard' },
  { name: '02_memories_explorer.png', view: 'memories', label: 'Native Memories Explorer' },
  { name: '10_memories_calendar.png', view: 'memories', label: 'Memories Calendar', tab: 'Calendar' },
  { name: '03_photo_toolkit_config.png', view: 'toolkit-config', label: 'Photo Processing Suite' },
  { name: '04_recap_video_config.png', view: 'recapper-config', label: 'Recap Video Generator' },
  { name: '05_activity_history.png', view: 'activity', label: 'Active Tasks & Generation History' },
];

function findBrowserBinary() {
  const candidates = [
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
    'C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium-browser',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge',
  ];

  for (const bin of candidates) {
    if (fs.existsSync(bin)) return bin;
  }
  return 'msedge';
}

async function isServerRunning(url) {
  try {
    const res = await fetch(url, { method: 'GET' });
    return res.status === 200 || res.status === 304;
  } catch {
    return false;
  }
}

async function main() {
  console.log('\n📸 [BeReal Studio] Launching Automated Screenshot Capture Suite...\n');

  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }

  let serverProcess = null;
  const baseUrl = process.env.SCREENSHOT_BASE_URL || 'http://localhost:1420';

  if (!(await isServerRunning(baseUrl))) {
    console.log(`⚡ Starting local Vite dev server for ${baseUrl}...`);
    const isWindows = process.platform === 'win32';
    const bunCmd = isWindows ? 'bun.cmd' : 'bun';
    serverProcess = spawn(bunCmd, ['run', 'dev'], {
      cwd: ROOT_DIR,
      stdio: 'pipe',
      shell: isWindows,
    });

    let attempts = 0;
    while (attempts < 30) {
      await new Promise((r) => setTimeout(r, 1000));
      attempts++;
      if (await isServerRunning(baseUrl)) {
        console.log(`✅ Local dev server ready on ${baseUrl}\n`);
        break;
      }
    }
  }

  const executablePath = findBrowserBinary();
  console.log(`🌐 Connecting to browser at: ${executablePath}`);

  const browser = await puppeteer.launch({
    executablePath,
    headless: true,
    defaultViewport: {
      width: 1280,
      height: 840,
      deviceScaleFactor: 2, // High-DPI Retina crispness
    },
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  });

  const page = await browser.newPage();
  page.on('console', (msg) => console.log('   [PAGE LOG]:', msg.text()));
  page.on('pageerror', (err) => console.log('   [PAGE ERROR]:', err.message));
  let captured = 0;

  for (let i = 0; i < VIEWS.length; i++) {
    const item = VIEWS[i];
    const outPath = path.resolve(SCREENSHOT_DIR, item.name);
    const targetUrl = `${baseUrl}?demo=1&view=${item.view}`;

    console.log(`[${i + 1}/${VIEWS.length}] Capturing ${item.label} (${item.name})...`);

    try {
      await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });

      // Wait for app header and view transition to complete
      try {
        await page.waitForSelector('.app-header', { timeout: 15000 });
      } catch {
        await page.reload({ waitUntil: 'domcontentloaded', timeout: 30000 });
        await page.waitForSelector('.app-header', { timeout: 15000 });
      }
      if (item.tab) {
        await page.waitForSelector('.segment-btn', { timeout: 8000 });
        const clicked = await page.evaluate((tab) => {
          const button = [...document.querySelectorAll('button.segment-btn')].find((element) => element.textContent?.trim() === tab);
          button?.click();
          return Boolean(button);
        }, item.tab);
        if (!clicked) throw new Error(`Could not find ${item.tab} tab`);
        await page.waitForSelector('.explorer-view-stage.is-active .day-cell.has-memory', { timeout: 8000 });
      }
      await page.waitForFunction(() => [...document.images]
        .filter((img) => img.getBoundingClientRect().width > 0)
        .every((img) => img.complete && img.naturalWidth > 0), { timeout: 15000 }).catch(() => {});
      if (item.tab === 'Calendar') {
        await page.evaluate(() => document.getElementById('calendar-day-2024-08-22')?.scrollIntoView({ block: 'center' }));
      }
      await new Promise((res) => setTimeout(res, 1200));

      await page.screenshot({ path: outPath, type: 'png' });

      const stats = fs.statSync(outPath);
      captured++;
      console.log(`   ✅ Successfully saved: ${item.name} (${(stats.size / 1024).toFixed(1)} KB)`);
    } catch (e) {
      console.warn(`   ⚠️ Warning: Could not capture ${item.name}:`, e.message);
    }
  }

  await browser.close();

  if (serverProcess) {
    console.log('🛑 Shutting down spawned Vite dev server...');
    if (process.platform === 'win32') {
      spawn('taskkill', ['/pid', serverProcess.pid.toString(), '/f', '/t']);
    } else {
      serverProcess.kill('SIGTERM');
    }
  }

  if (captured !== VIEWS.length) throw new Error(`Captured ${captured} of ${VIEWS.length} showcase screenshots.`);
  console.log(`\n🎉 All ${VIEWS.length} showcase screenshots successfully captured into docs/screenshots/!\n`);
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
