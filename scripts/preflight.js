import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const TAURI_DIR = path.resolve(ROOT_DIR, 'src-tauri');

const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  dim: '\x1b[2m'
};

function log(title, msg) {
  console.log(`${colors.cyan}[${title}]${colors.reset} ${msg}`);
}

function success(msg) {
  console.log(`  ${colors.green}✔${colors.reset} ${msg}`);
}

function fail(msg) {
  console.error(`  ${colors.red}✖${colors.reset} ${msg}`);
}

function run(cmd, cwd = ROOT_DIR) {
  console.log(`  ${colors.dim}$ ${cmd}${colors.reset}`);
  execSync(cmd, { cwd, stdio: 'inherit' });
}

const VALID_TAURI_CATEGORIES = [
  'Business', 'DeveloperTool', 'Education', 'Entertainment', 'Finance',
  'Games', 'Graphics and Design', 'HealthcareAndFitness', 'Lifestyle',
  'Medical', 'Music', 'News', 'Photography', 'Productivity',
  'Reference', 'SocialNetworking', 'Sports', 'Travel', 'Utilities',
  'Video', 'Weather'
];

async function main() {
  console.log(`\n${colors.bright}🚀 BeReal Studio — Pre-Release Preflight Verification${colors.reset}\n`);
  let errors = 0;

  // 1. Version Sync Check
  log('1/5', 'Checking version synchronization across manifests...');
  try {
    const pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT_DIR, 'package.json'), 'utf8'));
    const tauriConf = JSON.parse(fs.readFileSync(path.join(TAURI_DIR, 'tauri.conf.json'), 'utf8'));
    const cargoToml = fs.readFileSync(path.join(TAURI_DIR, 'Cargo.toml'), 'utf8');
    const cargoVersionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);

    const pkgVer = pkgJson.version;
    const tauriVer = tauriConf.version;
    const cargoVer = cargoVersionMatch ? cargoVersionMatch[1] : null;

    if (pkgVer === tauriVer && pkgVer === cargoVer) {
      success(`All manifests synchronized to version v${pkgVer}`);
    } else {
      fail(`Version mismatch detected: package.json (${pkgVer}), tauri.conf.json (${tauriVer}), Cargo.toml (${cargoVer})`);
      errors++;
    }

    // Category check
    const cat = tauriConf.bundle?.category;
    if (VALID_TAURI_CATEGORIES.includes(cat)) {
      success(`Valid Tauri bundler category: "${cat}"`);
    } else {
      fail(`Invalid Tauri bundle category "${cat}". Must be one of: ${VALID_TAURI_CATEGORIES.join(', ')}`);
      errors++;
    }

    // Windows NSIS check
    const nsisLangs = tauriConf.bundle?.windows?.nsis?.languages;
    if (nsisLangs && nsisLangs.includes('en-US')) {
      fail('NSIS languages cannot be "en-US"; must be "English"');
      errors++;
    } else {
      success('Windows NSIS packaging configuration verified');
    }

    // Icon files check
    const icons = tauriConf.bundle?.icon || [];
    let missingIcons = 0;
    for (const iconPath of icons) {
      const fullIconPath = path.join(TAURI_DIR, iconPath);
      if (!fs.existsSync(fullIconPath)) {
        fail(`Missing icon file: ${iconPath}`);
        missingIcons++;
      }
    }
    if (missingIcons === 0) {
      success(`All ${icons.length} Tauri icon assets verified`);
    } else {
      errors += missingIcons;
    }

    // README.md Version & Download links check
    const readmePath = path.join(ROOT_DIR, 'README.md');
    if (fs.existsSync(readmePath)) {
      const readmeContent = fs.readFileSync(readmePath, 'utf8');
      if (readmeContent.includes(`BeReal.Studio_${pkgVer}_x64-setup.exe`) &&
          readmeContent.includes(`BeReal Studio_${pkgVer}_x64-setup.exe`)) {
        success(`README.md download links and build artifact locations verified (v${pkgVer})`);
      } else {
        fail(`README.md contains outdated version references. Please update download links and build artifact paths to v${pkgVer}`);
        errors++;
      }
    }
  } catch (err) {
    fail(`Failed manifest checks: ${err.message}`);
    errors++;
  }

  // 2. Frontend Svelte Type Checking
  console.log('');
  log('2/5', 'Running Svelte type check (svelte-check)...');
  try {
    run('bun run check');
    success('Frontend diagnostics passed (0 errors)');
  } catch {
    fail('Frontend diagnostics failed');
    errors++;
  }

  // 3. Frontend Production Build
  console.log('');
  log('3/5', 'Compiling frontend production bundle (Vite + SvelteKit)...');
  try {
    run('bun run build');
    success('Frontend production bundle generated in build/');
  } catch {
    fail('Frontend production build failed');
    errors++;
  }

  // 4. Backend Rust Tests
  console.log('');
  log('4/5', 'Running Rust backend test suite (cargo test)...');
  try {
    run('cargo test', TAURI_DIR);
    success('All backend unit and benchmark tests passed');
  } catch {
    fail('Backend test suite failed');
    errors++;
  }

  // 5. Backend Rust Release Compilation Check
  console.log('');
  log('5/5', 'Verifying Rust release profile compilation (cargo check --release)...');
  try {
    run('cargo check --release', TAURI_DIR);
    success('Backend release check passed');
  } catch {
    fail('Backend release check failed');
    errors++;
  }

  // Summary
  console.log(`\n------------------------------------------------------------`);
  if (errors === 0) {
    console.log(`${colors.green}${colors.bright}✔ ALL PREFLIGHT CHECKS PASSED! Codebase is ready for release.${colors.reset}\n`);
    process.exit(0);
  } else {
    console.log(`${colors.red}${colors.bright}✖ PREFLIGHT FAILED: ${errors} error(s) found. Please resolve before releasing.${colors.reset}\n`);
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
