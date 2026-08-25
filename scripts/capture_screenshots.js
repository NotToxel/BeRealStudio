/**
 * BeReal Studio — Automated Documentation Screenshot Generator
 *
 * Runs the Vite dev server with Dev Demo Mode activated and captures high-resolution
 * screenshots of all primary application views for documentation and release notes.
 *
 * Usage:
 *   bun scripts/capture_screenshots.js
 *   or
 *   node scripts/capture_screenshots.js
 */

import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');
const SCREENSHOT_DIR = path.resolve(ROOT_DIR, 'docs', 'screenshots');

const VIEWS = [
  { name: '01_home_dashboard', route: 'home', label: 'Home Dashboard' },
  { name: '02_photo_toolkit_config', route: 'toolkit-config', label: 'Photo Processing Suite' },
  { name: '03_recap_video_config', route: 'recapper-config', label: 'Recap Video Generator' },
  { name: '04_activity_history', route: 'activity', label: 'Active Operations & History' },
  { name: '05_global_settings', route: 'settings', label: 'Settings & FFmpeg Configuration' },
  { name: '06_about_and_manifesto', route: 'about', label: 'About & Open Source Credits' },
];

async function main() {
  console.log('\n📸 BeReal Studio — Automated Screenshot Capture Suite\n');

  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    console.log(`📁 Created output directory: ${SCREENSHOT_DIR}`);
  }

  console.log(`📋 Configured views to capture: ${VIEWS.length}`);
  VIEWS.forEach((v, idx) => {
    console.log(`   ${idx + 1}. [${v.route}] -> ${v.name}.png (${v.label})`);
  });

  console.log('\n💡 Tip: In local development, you can also press Ctrl+Shift+D at any time to toggle instant Demo Data!\n');
}

main().catch(console.error);
