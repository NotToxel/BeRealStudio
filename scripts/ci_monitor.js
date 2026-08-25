// scripts/ci_monitor.js
async function checkStatus() {
  try {
    const res = await fetch('https://github.com/NotToxel/BeRealStudio/releases/tag/v2.0.0', {
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'
      }
    });
    if (res.status === 404) {
      console.log(`[${new Date().toLocaleTimeString()}] Release page not yet created. Waiting for workflow...`);
      return false;
    }
    const html = await res.text();
    
    // Find all release download asset links
    const assetRegex = /\/NotToxel\/BeRealStudio\/releases\/download\/v2\.0\.0\/([a-zA-Z0-9_\-\.]+)/g;
    const matches = new Set();
    let m;
    while ((m = assetRegex.exec(html)) !== null) {
      if (!m[1].endsWith('.zip') && !m[1].endsWith('.tar.gz') || m[1].includes('dmg') || m[1].includes('exe') || m[1].includes('msi') || m[1].includes('AppImage') || m[1].includes('deb')) {
        matches.add(m[1]);
      }
    }

    console.log(`[${new Date().toLocaleTimeString()}] Found ${matches.size} release binaries in GitHub Release v2.0.0:`);
    for (const file of matches) {
      console.log(`  ✅ ${file}`);
    }

    // Check if all major desktop binaries are present
    const hasWindows = Array.from(matches).some(f => f.endsWith('.exe') || f.endsWith('.msi'));
    const hasMac = Array.from(matches).some(f => f.endsWith('.dmg'));
    const hasLinux = Array.from(matches).some(f => f.endsWith('.AppImage') || f.endsWith('.deb'));

    if (hasWindows && hasMac && hasLinux) {
      console.log('\n🎉 ALL TARGET PLATFORM BINARIES (Windows, macOS, Linux) SUCCESSFULLY DELIVERED ON GITHUB!');
      return true;
    } else {
      console.log(`[${new Date().toLocaleTimeString()}] Waiting for remaining platform binaries (Windows: ${hasWindows}, Mac: ${hasMac}, Linux: ${hasLinux})...`);
    }
    return false;
  } catch (err) {
    console.error(`[${new Date().toLocaleTimeString()}] Check error:`, err.message);
    return false;
  }
}

async function loop() {
  console.log('Starting BeReal Studio Release Delivery Monitor...');
  while (true) {
    const complete = await checkStatus();
    if (complete) {
      console.log('\nSUCCESS: Release v2.0.0 is fully published and downloadable!');
      break;
    }
    await new Promise(r => setTimeout(r, 45000));
  }
}

loop();
