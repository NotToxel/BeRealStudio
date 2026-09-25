import fs from 'node:fs';
import path from 'node:path';
import puppeteer from 'puppeteer-core';

const browserPath = [
  'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
  'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
  'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
].find((candidate) => fs.existsSync(candidate));

if (!browserPath) throw new Error('Edge or Chrome is required for the map smoke check.');

const browser = await puppeteer.launch({ executablePath: browserPath, headless: true, args: ['--no-sandbox'] });
try {
  const baseUrl = process.env.MAP_VERIFY_BASE_URL || 'http://127.0.0.1:1420';
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 850, deviceScaleFactor: 1 });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('requestfailed', (request) => { if (request.url().includes('world-countries')) errors.push(`World data request failed: ${request.failure()?.errorText}`); });
  await page.goto(`${baseUrl}/?demo=1&view=memories`, { waitUntil: 'networkidle2' });
  await page.waitForSelector('.segment-btn', { timeout: 10000 });
  const clickedMap = await page.evaluate(() => {
    const button = [...document.querySelectorAll('button.segment-btn')].find((element) => element.textContent?.trim() === 'Map');
    button?.click();
    return Boolean(button);
  });
  if (!clickedMap) throw new Error('Map tab was not found.');
  await page.waitForSelector('.maplibregl-canvas', { timeout: 15000 });
  await new Promise((resolve) => setTimeout(resolve, 1800));
  const result = await page.evaluate(() => ({
    mapped: document.querySelector('.map-topline')?.textContent?.trim(),
    resultRows: document.querySelectorAll('.map-results .result-row').length,
    rowCameraImages: document.querySelectorAll('.map-results .result-row .map-memory-thumbnail img').length,
    pinCount: document.querySelectorAll('.memory-map-pin').length,
    clusterCount: document.querySelectorAll('.memory-map-cluster').length,
    canvasWidth: document.querySelector('.maplibregl-canvas')?.clientWidth,
    canvasHeight: document.querySelector('.maplibregl-canvas')?.clientHeight,
  }));
  const screenshot = path.resolve('docs/screenshots/06_memories_map.png');
  await page.screenshot({ path: screenshot });
  if (errors.length || !result.canvasWidth || !result.mapped?.includes('5 mapped') || !result.resultRows || result.rowCameraImages < 10) {
    throw new Error(JSON.stringify({ result, errors }));
  }
  const thumbnailLayout = await page.evaluate(async () => {
    const first = document.querySelector('.result-row .map-memory-thumbnail');
    const before = { main: first.querySelector('.main-camera')?.src, inset: first.querySelector('.inset-camera')?.src };
    const frame = first.getBoundingClientRect();
    const inset = first.querySelector('.inset-frame').getBoundingClientRect();
    const { globalPerspective } = await import('/src/lib/memoriesStore.ts');
    globalPerspective.set('secondary');
    await new Promise((resolve) => setTimeout(resolve, 50));
    const after = { main: first.querySelector('.main-camera')?.src, inset: first.querySelector('.inset-camera')?.src };
    globalPerspective.set('primary');
    return { aspect: frame.width / frame.height, frameWidth: frame.width, insetWidth: inset.width, insetFraction: inset.width / frame.width, insetTop: (inset.top - frame.top) / frame.height, swapped: after.main === before.inset && after.inset === before.main };
  });
  if (Math.abs(thumbnailLayout.aspect - .75) > .02 || thumbnailLayout.frameWidth < 90 || Math.abs(thumbnailLayout.insetFraction - .42) > .02 || thumbnailLayout.insetWidth < 35 || Math.abs(thumbnailLayout.insetTop - .0378) > .02 || !thumbnailLayout.swapped) {
    throw new Error(`Map thumbnail is too small or has the wrong camera layout: ${JSON.stringify(thumbnailLayout)}`);
  }
  const transitionSamples = await page.evaluate(async () => {
    document.querySelector('.memory-map-cluster')?.click();
    const samples = [];
    for (const elapsed of [100, 200, 300, 450, 650]) {
      await new Promise((resolve) => setTimeout(resolve, elapsed - (samples.at(-1)?.elapsed ?? 0)));
      samples.push({ elapsed, pins: document.querySelectorAll('.memory-map-pin').length, clusters: document.querySelectorAll('.memory-map-cluster').length });
    }
    return samples;
  });
  await new Promise((resolve) => setTimeout(resolve, 450));
  const expanded = await page.evaluate(() => ({
    pins: document.querySelectorAll('.memory-map-pin').length,
    clusters: document.querySelectorAll('.memory-map-cluster').length,
  }));
  if (!transitionSamples.some((sample) => sample.elapsed <= 450 && sample.pins > 0)) {
    throw new Error(`Photo pins did not split during zoom: ${JSON.stringify(transitionSamples)}`);
  }
  if (!expanded.pins) throw new Error(`Cluster did not expand to photo pins: ${JSON.stringify(expanded)}`);
  const mapPinLayout = await page.evaluate(() => {
    const pin = document.querySelector('.memory-map-pin');
    const frame = pin.getBoundingClientRect();
    const inset = pin.querySelector('.memory-map-pin-pip')?.getBoundingClientRect();
    return { width: frame.width, height: frame.height, insetWidth: inset?.width, insetHeight: inset?.height, pointerBottom: getComputedStyle(pin, '::after').bottom };
  });
  if (Math.abs(mapPinLayout.width / mapPinLayout.height - .75) > .01 || mapPinLayout.width < 90 || (mapPinLayout.insetWidth ?? 0) < 34 || mapPinLayout.pointerBottom !== '-13px') {
    throw new Error(`Map photo pin has the wrong BeReal proportions: ${JSON.stringify(mapPinLayout)}`);
  }
  const markerBeforePan = await page.evaluate(() => {
    const pin = document.querySelector('.memory-map-pin');
    return { position: getComputedStyle(pin).position, transform: pin.style.transform };
  });
  if (markerBeforePan.position !== 'absolute') throw new Error(`Pin lost MapLibre anchoring: ${JSON.stringify(markerBeforePan)}`);
  const canvas = await page.$('.maplibregl-canvas');
  const rect = await canvas.boundingBox();
  await page.mouse.move(rect.x + rect.width * .44, rect.y + rect.height * .55);
  await page.mouse.down();
  await page.mouse.move(rect.x + rect.width * .44 + 60, rect.y + rect.height * .55 + 30, { steps: 8 });
  await page.mouse.up();
  await new Promise((resolve) => setTimeout(resolve, 750));
  const markerAfterPan = await page.evaluate(() => document.querySelector('.memory-map-pin')?.style.transform);
  if (!markerAfterPan || markerAfterPan === markerBeforePan.transform) throw new Error('Photo pin did not move with the map pan.');
  const detailScreenshot = path.resolve('docs/screenshots/07_memories_map_detail.png');
  await page.screenshot({ path: detailScreenshot });
  await page.evaluate(() => document.querySelector('.memory-map-pin')?.click());
  const selected = await page.evaluate(() => document.querySelector('.selected-head')?.textContent?.trim() || '');
  if (!selected.includes('memory')) throw new Error(`Photo pin did not select a memory: ${selected}`);
  await page.evaluate(() => document.querySelector('.group-gallery .gallery-card')?.click());
  const feedOpened = await page.evaluate(() => Boolean(document.querySelector('.feed-modal-backdrop')));
  if (!feedOpened) throw new Error('Selected map memory did not open the feed.');
  await page.goto(`${baseUrl}/?demo=1&view=memories`, { waitUntil: 'networkidle2' });
  await page.evaluate(() => [...document.querySelectorAll('button.segment-btn')].find((element) => element.textContent?.trim() === 'Map')?.click());
  await page.waitForSelector('.memory-map-cluster');
  await page.evaluate(() => document.querySelector('.memory-map-cluster')?.click());
  await page.waitForSelector('.memory-map-pin');
  const wheelSamples = [];
  const wheelCanvas = await page.$('.maplibregl-canvas');
  const wheelRect = await wheelCanvas.boundingBox();
  await page.mouse.move(wheelRect.x + wheelRect.width * .5, wheelRect.y + wheelRect.height * .35);
  for (let step = 0; step < 5; step++) {
    await page.mouse.wheel({ deltaY: 220 });
    await new Promise((resolve) => setTimeout(resolve, 130));
    wheelSamples.push(await page.evaluate(() => ({ pins: document.querySelectorAll('.memory-map-pin').length, clusters: document.querySelectorAll('.memory-map-cluster').length })));
  }
  if (!wheelSamples.some((sample) => sample.pins === 0)) throw new Error(`Photo pins did not combine while scrolling: ${JSON.stringify(wheelSamples)}`);
  await page.goto(`${baseUrl}/?demo=1&view=memories`, { waitUntil: 'networkidle2' });
  await page.evaluate(() => [...document.querySelectorAll('button.segment-btn')].find((element) => element.textContent?.trim() === 'Map')?.click());
  await page.waitForSelector('.maplibregl-canvas');
  await page.evaluate(async () => {
    const { explorerData } = await import('/src/lib/memoriesStore.ts');
    explorerData.update((data) => {
      if (!data) return data;
      const london = data.memories.find((memory) => memory.city === 'London');
      const dense = Array.from({ length: 120 }, (_, index) => ({
        ...london, id: `dense-${index}`, suburb: 'Shoreditch',
        location: { ...london.location },
        takenAt: new Date(Date.UTC(2024, 0, 1 + index)).toISOString(),
        dateFormatted: `Dense ${index + 1}`,
      }));
      return { ...data, memories: [...data.memories, ...dense], totalCount: data.memories.length + dense.length };
    });
  });
  await new Promise((resolve) => setTimeout(resolve, 900));
  const densityScreenshot = path.resolve('docs/screenshots/09_memories_map_density.png');
  await page.screenshot({ path: densityScreenshot });
  let clusterGalleryCount = 0;
  for (let attempt = 0; attempt < 8 && clusterGalleryCount < 120; attempt++) {
    const clicked = await page.evaluate(() => {
      const densePin = [...document.querySelectorAll('.memory-map-pin')].find((pin) => Number(pin.querySelector('.memory-map-pin-count')?.textContent) >= 120);
      if (densePin) { densePin.click(); return true; }
      const cluster = [...document.querySelectorAll('.memory-map-cluster')].sort((a, b) => Number(b.textContent) - Number(a.textContent))[0];
      cluster?.click();
      return Boolean(cluster);
    });
    if (!clicked) break;
    await new Promise((resolve) => setTimeout(resolve, 850));
    clusterGalleryCount = await page.evaluate(() => Number(document.querySelector('.selected-head')?.textContent?.match(/\d+/)?.[0] ?? 0));
  }
  if (clusterGalleryCount < 120) throw new Error(`Dense map location did not open its photo gallery: ${clusterGalleryCount}; ${errors.join(' | ')}`);
  await page.evaluate(() => [...document.querySelectorAll('.panel-tabs button')].find((button) => button.textContent?.trim() === 'Places')?.click());
  const countries = await page.evaluate(() => [...document.querySelectorAll('.place-row')].map((row) => row.textContent?.trim()));
  const placeCameraImages = await page.evaluate(() => document.querySelectorAll('.place-row .map-memory-thumbnail img').length);
  if (!countries.some((country) => country.includes('United Kingdom'))) throw new Error(`Countries missing: ${countries}`);
  if (placeCameraImages < 2) throw new Error('Place cover does not show both BeReal cameras.');
  await page.evaluate(() => document.querySelector('.place-row')?.click());
  const cities = await page.evaluate(() => [...document.querySelectorAll('.place-row')].map((row) => row.textContent?.trim()));
  if (!cities.some((city) => city.includes('London'))) throw new Error(`Cities missing: ${cities}`);
  await page.evaluate(() => [...document.querySelectorAll('.place-row')].find((row) => row.textContent?.includes('London'))?.click());
  const suburbs = await page.evaluate(() => [...document.querySelectorAll('.place-row')].map((row) => row.textContent?.trim()));
  if (!suburbs.some((suburb) => suburb.includes('Shoreditch'))) throw new Error(`Suburbs missing: ${suburbs}`);
  await page.evaluate(() => [...document.querySelectorAll('.place-row')].find((row) => row.textContent?.includes('Shoreditch'))?.click());
  const denseGallery = await page.evaluate(() => ({
    title: document.querySelector('.selected-head')?.textContent?.trim(),
    cards: document.querySelectorAll('.group-gallery .gallery-card').length,
    showMore: document.querySelector('.show-more')?.textContent?.trim(),
    photoWidth: document.querySelector('.gallery-card .map-memory-thumbnail')?.getBoundingClientRect().width,
    insetWidth: document.querySelector('.gallery-card .inset-frame')?.getBoundingClientRect().width,
  }));
  if (!denseGallery.title?.includes('120 memories') || denseGallery.cards !== 40 || !denseGallery.showMore || denseGallery.photoWidth < 170 || denseGallery.insetWidth < 70) {
    throw new Error(`Dense place gallery failed: ${JSON.stringify(denseGallery)}`);
  }
  if (await page.evaluate(() => document.querySelectorAll('.gallery-card .map-memory-thumbnail img').length) < 80) {
    throw new Error('Gallery cards do not show both BeReal cameras.');
  }
  await new Promise((resolve) => setTimeout(resolve, 1000));
  const denseScreenshot = path.resolve('docs/screenshots/08_memories_map_dense_gallery.png');
  await page.screenshot({ path: denseScreenshot });
  await page.evaluate(() => document.querySelector('.show-more')?.click());
  if (await page.evaluate(() => document.querySelectorAll('.group-gallery .gallery-card').length) !== 80) throw new Error('Dense gallery pagination failed.');
  console.log(JSON.stringify({ result, thumbnailLayout, transitionSamples, wheelSamples, expanded, mapPinLayout, markerBeforePan, markerAfterPan, selected, feedOpened, clusterGalleryCount, countries, cities, suburbs, denseGallery, screenshot, detailScreenshot, denseScreenshot, densityScreenshot, errors }, null, 2));
} finally {
  await browser.close();
}
