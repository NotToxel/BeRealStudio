<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as maplibregl from 'maplibre-gl';
  import type { GeoJSONSource, Marker } from 'maplibre-gl';
  import mapWorkerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { explorerFilter, filteredMemories, getMediaDataUrl, getSafeImageSrc, openFeedAt } from '$lib/memoriesStore';
  import { getLocatedMemories, getMemoryCover, getMemoryPlaces, isInsideMapBounds, type LocatedMemory, type PlaceLevel } from '$lib/memoriesMap';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Layers from 'lucide-svelte/icons/layers';
  import Scan from 'lucide-svelte/icons/scan';
  import X from 'lucide-svelte/icons/circle-x';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';
  import MemoryMapThumbnail from './MemoryMapThumbnail.svelte';

  type PointGroup = { key: string; items: LocatedMemory[]; latitude: number; longitude: number };
  const OFFLINE_STYLE: maplibregl.StyleSpecification = {
    version: 8,
    name: 'BeReal Studio offline overview',
    sources: { countries: { type: 'geojson', data: '/world-countries.geojson' } },
    layers: [
      { id: 'sea', type: 'background', paint: { 'background-color': '#121b2b' } },
      { id: 'land', type: 'fill', source: 'countries', paint: { 'fill-color': '#10415b' } },
      { id: 'boundaries', type: 'line', source: 'countries', paint: { 'line-color': '#597486', 'line-opacity': 0.36, 'line-width': 0.7 } },
    ],
  };
  const KEY_STORAGE = 'bereal_maptiler_key';
  const DETAIL_STORAGE = 'bereal_map_detail_enabled';
  const MAX_PINS = 12;

  let container: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let markers = new Map<string, Marker>();
  let clusterMarkers = new Map<number, Marker>();
  let groupedPoints = new Map<string, PointGroup>();
  let visibleItems: LocatedMemory[] = [];
  let selectedGroup: PointGroup | null = null;
  let panelMode: 'nearby' | 'places' | 'group' = 'nearby';
  let visibleGroupCount = 40;
  let panelOpen = true;
  let showTileSettings = false;
  let keyInput = '';
  let tileKey = '';
  let tileError = '';
  let detailedMap = false;
  let mapReady = false;
  let clusterEventsInstalled = false;
  let lastViewportUpdate = 0;
  let viewportFrame = 0;
  let markerFrame = 0;
  let lastMarkerUpdate = 0;
  let resizeObserver: ResizeObserver | undefined;

  $: located = getLocatedMemories($filteredMemories);
  $: placeLevel = ($explorerFilter.selectedCountry === 'all' ? 'country' : $explorerFilter.selectedCity === 'all' ? 'city' : 'suburb') as PlaceLevel;
  $: places = getMemoryPlaces(located, placeLevel);
  $: unmappedCount = $filteredMemories.length - located.length;
  $: if (map && mapReady && located) updateSource();

  function makeGroups(items: LocatedMemory[]): Map<string, PointGroup> {
    const groups = new Map<string, PointGroup>();
    for (const item of items) {
      const key = `${item.latitude.toFixed(5)},${item.longitude.toFixed(5)}`;
      const group = groups.get(key);
      if (group) group.items.push(item);
      else groups.set(key, { key, items: [item], latitude: item.latitude, longitude: item.longitude });
    }
    return groups;
  }

  function toFeatureCollection(): GeoJSON.FeatureCollection {
    return {
      type: 'FeatureCollection',
      features: [...groupedPoints.values()].map((group) => ({
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [group.longitude, group.latitude] },
        properties: { key: group.key, memoryCount: group.items.length },
      })),
    };
  }

  function toHeatFeatureCollection(): GeoJSON.FeatureCollection {
    return {
      type: 'FeatureCollection',
      features: [...groupedPoints.values()].map((group) => ({
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [group.longitude, group.latitude] },
        properties: { memoryCount: group.items.length },
      })),
    };
  }

  function installLayers() {
    if (!map) return;
    map.addSource('memories', {
      type: 'geojson', data: toFeatureCollection(), cluster: true, maxzoom: 20,
      clusterRadius: 80, clusterMaxZoom: 19,
      clusterProperties: { memoryTotal: ['+', ['get', 'memoryCount']] },
    });
    map.addSource('memory-heat-data', { type: 'geojson', data: toHeatFeatureCollection() });
    map.addLayer({
      id: 'memory-heat', type: 'heatmap', source: 'memory-heat-data',
      maxzoom: 12,
      paint: {
        'heatmap-weight': ['interpolate', ['linear'], ['get', 'memoryCount'], 1, 0.45, 3, 0.85, 10, 1.5, 50, 2.6, 200, 4],
        'heatmap-intensity': ['interpolate', ['linear'], ['zoom'], 0, 0.9, 6, 1.25, 11, 1.5],
        'heatmap-radius': ['interpolate', ['linear'], ['zoom'], 0, 36, 3, 60, 6, 90, 11, 106],
        'heatmap-opacity': ['interpolate', ['linear'], ['zoom'], 0, 0.65, 6, 0.85, 9, 0.55, 12, 0],
        'heatmap-color': ['interpolate', ['linear'], ['heatmap-density'], 0, 'rgba(19,104,171,0)', 0.18, '#167db4', 0.38, '#20b5d4', 0.62, '#77d9df', 0.82, '#f4dd6d', 1, '#fff2b0'],
      },
    });
    map.addLayer({
      id: 'memory-clusters', type: 'circle', source: 'memories', filter: ['has', 'point_count'],
      paint: {
        'circle-color': '#f5f9ff',
        'circle-radius': ['step', ['get', 'memoryTotal'], 20, 10, 24, 50, 29, 200, 34],
        'circle-stroke-color': '#135477', 'circle-stroke-width': 3,
      },
    });
    map.addLayer({
      id: 'memory-single-hit', type: 'circle', source: 'memories', filter: ['!', ['has', 'point_count']],
      paint: { 'circle-radius': 6, 'circle-color': '#b8e8ff', 'circle-opacity': 0.9, 'circle-stroke-color': '#102d40', 'circle-stroke-width': 2 },
    });
    map.setLayoutProperty('memory-heat', 'visibility', located.length >= 2 ? 'visible' : 'none');
    if (!clusterEventsInstalled) {
      clusterEventsInstalled = true;
      map.on('click', 'memory-clusters', (event) => {
        if (!map) return;
        const feature = map.queryRenderedFeatures(event.point, { layers: ['memory-clusters'] })[0];
        const clusterId = feature?.properties?.cluster_id;
        if (typeof clusterId !== 'number') return;
        const source = map.getSource('memories') as GeoJSONSource;
        openCluster(source, clusterId, feature.geometry.type === 'Point' ? feature.geometry.coordinates as [number, number] : [0, 0]);
      });
      map.on('click', 'memory-single-hit', (event) => {
        if (!map) return;
        const feature = map.queryRenderedFeatures(event.point, { layers: ['memory-single-hit'] })[0];
        const group = groupedPoints.get(String(feature?.properties?.key ?? ''));
        if (group) selectGroup(group);
      });
      map.on('mouseenter', 'memory-clusters', () => { if (map) map.getCanvas().style.cursor = 'pointer'; });
      map.on('mouseleave', 'memory-clusters', () => { if (map) map.getCanvas().style.cursor = ''; });
    }
    updateViewport();
  }

  function updateSource() {
    if (!map) return;
    groupedPoints = makeGroups(located);
    const source = map.getSource('memories') as GeoJSONSource | undefined;
    if (source) source.setData(toFeatureCollection());
    const heatSource = map.getSource('memory-heat-data') as GeoJSONSource | undefined;
    if (heatSource) heatSource.setData(toHeatFeatureCollection());
    if (map.getLayer('memory-heat')) map.setLayoutProperty('memory-heat', 'visibility', located.length >= 2 ? 'visible' : 'none');
    for (const marker of markers.values()) marker.remove();
    markers.clear();
    for (const marker of clusterMarkers.values()) marker.remove();
    clusterMarkers.clear();
    if (selectedGroup) {
      const ids = new Set(located.map((item) => item.memory.id));
      const items = selectedGroup.items.filter((item) => ids.has(item.memory.id));
      selectedGroup = items.length ? { ...selectedGroup, items } : null;
      if (!selectedGroup && panelMode === 'group') panelMode = 'nearby';
    }
    updateViewport();
  }

  function updateViewport() {
    if (!map) return;
    const bounds = map.getBounds();
    visibleItems = located
      .filter((item) => isInsideMapBounds(item, bounds.getWest(), bounds.getSouth(), bounds.getEast(), bounds.getNorth()))
      .sort((a, b) => b.memory.takenAt.localeCompare(a.memory.takenAt));
  }

  function queueViewportUpdate() {
    if (viewportFrame) return;
    viewportFrame = requestAnimationFrame(() => {
      viewportFrame = 0;
      const now = performance.now();
      if (now - lastViewportUpdate < 85) return;
      lastViewportUpdate = now;
      updateViewport();
    });
  }

  function queueMarkerUpdate() {
    if (markerFrame) return;
    markerFrame = requestAnimationFrame(() => {
      markerFrame = 0;
      const now = performance.now();
      if (now - lastMarkerUpdate < 55) return;
      lastMarkerUpdate = now;
      renderMarkers();
    });
  }

  function addImage(parent: HTMLElement, path: string | undefined, className: string) {
    if (!path) return;
    const image = document.createElement('img');
    image.className = className;
    image.alt = '';
    image.loading = 'lazy';
    image.src = getSafeImageSrc(path);
    image.addEventListener('error', () => {
      if (image.dataset.fallbackAttempted) return;
      image.dataset.fallbackAttempted = 'true';
      getMediaDataUrl(path).then((url) => { if (image.isConnected) image.src = url; }).catch(() => {});
    });
    parent.append(image);
  }

  function createPin(group: PointGroup): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'memory-map-pin';
    if (group.items.length > 1) button.classList.add('is-group');
    button.setAttribute('aria-label', group.items.length === 1
      ? `Memory from ${group.items[0].memory.dateFormatted}`
      : `${group.items.length} memories at this location`);
    const photo = document.createElement('span');
    photo.className = 'memory-map-pin-photo';
    const first = group.items[0].memory;
    addImage(photo, getMemoryCover(first), 'memory-map-pin-main');
    if (first.secondaryPath && first.secondaryPath !== getMemoryCover(first) && !/\.(mp4|mov|m4v|webm)(\?|$)/i.test(first.secondaryPath)) {
      addImage(photo, first.secondaryPath, 'memory-map-pin-pip');
    }
    button.append(photo);
    if (group.items.length > 1) {
      const badge = document.createElement('span');
      badge.className = 'memory-map-pin-count';
      badge.textContent = String(group.items.length);
      button.append(badge);
    }
    button.addEventListener('click', (event) => {
      event.stopPropagation();
      selectGroup(group);
    });
    return button;
  }

  function selectGroup(group: PointGroup) {
    selectedGroup = { ...group, items: [...group.items].sort((a, b) => b.memory.takenAt.localeCompare(a.memory.takenAt)) };
    visibleGroupCount = 40;
    panelOpen = true;
    panelMode = 'group';
  }

  async function openCluster(source: GeoJSONSource, id: number, coordinates: [number, number]) {
    try {
      const zoom = await source.getClusterExpansionZoom(id);
      if (!map) return;
      if (zoom <= (detailedMap ? 15 : 6) && map.getZoom() < (detailedMap ? 15 : 6)) {
        map.easeTo({ center: coordinates, zoom: Math.min(zoom, detailedMap ? 16 : 6), offset: [mapPanelOffset(), 0], duration: 240 });
        return;
      }
      const leaves = await source.getClusterLeaves(id, 10000, 0);
      const items = leaves.flatMap((feature) => groupedPoints.get(String(feature.properties?.key ?? ''))?.items ?? []);
      if (items.length) selectGroup({ key: `cluster-${id}`, items, longitude: coordinates[0], latitude: coordinates[1] });
    } catch { /* Source can be replaced while the map is moving. */ }
  }

  function mapPanelOffset(): number {
    return panelOpen ? (container.clientWidth <= 1100 ? -175 : -210) : 0;
  }

  function renderMarkers() {
    if (!map || !map.getLayer('memory-single-hit')) return;
    const canvas = map.getCanvas();
    const clusters = map.queryRenderedFeatures([[0, 0], [canvas.clientWidth, canvas.clientHeight]], { layers: ['memory-clusters'] });
    const clusterIds = new Set<number>();
    for (const feature of clusters) {
      const id = feature.properties?.cluster_id;
      if (typeof id !== 'number' || feature.geometry.type !== 'Point') continue;
      clusterIds.add(id);
      if (clusterMarkers.has(id)) continue;
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'memory-map-cluster';
      const count = Number(feature.properties?.memoryTotal ?? feature.properties?.point_count ?? 0);
      button.textContent = count >= 1000 ? `${(count / 1000).toFixed(count < 10000 ? 1 : 0)}k` : String(count);
      button.style.setProperty('--cluster-size', `${count >= 200 ? 64 : count >= 50 ? 56 : count >= 10 ? 48 : 42}px`);
      button.setAttribute('aria-label', `Explore ${count} memories in this area`);
      const coordinates = feature.geometry.coordinates as [number, number];
      button.addEventListener('click', () => {
        const source = map?.getSource('memories') as GeoJSONSource | undefined;
        if (source) openCluster(source, id, coordinates);
      });
      clusterMarkers.set(id, new maplibregl.Marker({ element: button, anchor: 'center' }).setLngLat(coordinates).addTo(map));
    }
    for (const [id, marker] of clusterMarkers) {
      if (!clusterIds.has(id)) { marker.remove(); clusterMarkers.delete(id); }
    }
    const features = map.queryRenderedFeatures([[0, 0], [canvas.clientWidth, canvas.clientHeight]], { layers: ['memory-single-hit'] });
    const keys = [...new Set(features.map((feature) => String(feature.properties?.key ?? '')))].filter(Boolean).slice(0, MAX_PINS);
    const next = new Set(keys);
    for (const [key, marker] of markers) {
      if (!next.has(key)) { marker.remove(); markers.delete(key); }
    }
    for (const key of keys) {
      if (markers.has(key)) continue;
      const group = groupedPoints.get(key);
      if (!group) continue;
      const marker = new maplibregl.Marker({ element: createPin(group), anchor: 'bottom', offset: [0, -13] })
        .setLngLat([group.longitude, group.latitude]).addTo(map);
      markers.set(key, marker);
    }
  }

  function fitItems(items: LocatedMemory[]) {
    if (!map || !items.length) return;
    if (items.length === 1) {
      map.easeTo({ center: [items[0].longitude, items[0].latitude], zoom: detailedMap ? 8.5 : 4.5, offset: [mapPanelOffset(), 0], duration: 300 });
      return;
    }
    const longitudes = items.map((item) => (item.longitude + 360) % 360).sort((a, b) => a - b);
    let gapIndex = 0;
    let largestGap = -1;
    for (let i = 0; i < longitudes.length; i++) {
      const next = i === longitudes.length - 1 ? longitudes[0] + 360 : longitudes[i + 1];
      const gap = next - longitudes[i];
      if (gap > largestGap) { largestGap = gap; gapIndex = i; }
    }
    let west = longitudes[(gapIndex + 1) % longitudes.length];
    let east = longitudes[gapIndex] + (gapIndex === longitudes.length - 1 ? 0 : 360);
    if (east < west) east += 360;
    if (west > 180) { west -= 360; east -= 360; }
    const south = Math.min(...items.map((item) => item.latitude));
    const north = Math.max(...items.map((item) => item.latitude));
    if (west === east && south === north) {
      map.easeTo({ center: [items[0].longitude, items[0].latitude], zoom: detailedMap ? 8.5 : 4.5, offset: [mapPanelOffset(), 0], duration: 300 });
      return;
    }
    map.fitBounds([[west, south], [east, north]], {
      padding: { top: 72, bottom: 72, left: 72, right: panelOpen ? (container.clientWidth <= 1100 ? 396 : 456) : 72 },
      maxZoom: detailedMap ? 10 : 4.5,
      duration: 350,
    });
  }

  function focusMemory(item: LocatedMemory) {
    if (!map) return;
    const group = groupedPoints.get(`${item.latitude.toFixed(5)},${item.longitude.toFixed(5)}`);
    if (group) selectGroup(group);
    map.easeTo({ center: [item.longitude, item.latitude], zoom: Math.max(map.getZoom(), detailedMap ? 9 : 4.5), offset: [mapPanelOffset(), 0], duration: 300 });
  }

  function clearPlaceTo(level: 'country' | 'city' | 'suburb') {
    explorerFilter.update((filter) => ({ ...filter,
      selectedCountry: level === 'country' ? 'all' : filter.selectedCountry,
      selectedCity: level === 'suburb' ? filter.selectedCity : 'all',
      selectedSuburb: 'all',
    }));
  }

  function choosePlace(place: { name: string; value: string | null; memories: LocatedMemory[] }) {
    fitItems(place.memories);
    if (!place.value || placeLevel === 'suburb') {
      if (place.value) explorerFilter.update((filter) => ({ ...filter, selectedSuburb: place.value! }));
      selectGroup({ key: `place-${placeLevel}-${place.name}`, items: place.memories, latitude: place.memories[0].latitude, longitude: place.memories[0].longitude });
      return;
    }
    explorerFilter.update((filter) => ({ ...filter,
      selectedCountry: placeLevel === 'country' ? place.value! : filter.selectedCountry,
      selectedCity: placeLevel === 'city' ? place.value! : 'all',
      selectedSuburb: 'all',
    }));
  }

  async function enableDetail() {
    if (!map) return;
    const key = keyInput.trim();
    if (!key) { tileError = 'Enter a MapTiler API key to load detailed tiles.'; return; }
    tileError = '';
    const styleUrl = `https://api.maptiler.com/maps/streets-v4-dark/style.json?key=${encodeURIComponent(key)}`;
    try {
      const response = await fetch(styleUrl);
      if (!response.ok) throw new Error(`MapTiler returned ${response.status}. Check the key and its usage limits.`);
      localStorage.setItem(KEY_STORAGE, key);
      localStorage.setItem(DETAIL_STORAGE, 'true');
      tileKey = key;
      map.setMaxZoom(18);
      map.setStyle(styleUrl);
      detailedMap = true;
      showTileSettings = false;
    } catch (error) {
      tileError = error instanceof Error ? error.message : 'Could not load detailed map tiles.';
      showTileSettings = true;
    }
  }

  function useOfflineMap() {
    if (!map) return;
    map.setStyle(OFFLINE_STYLE);
    map.setMaxZoom(6);
    localStorage.setItem(DETAIL_STORAGE, 'false');
    detailedMap = false;
    showTileSettings = false;
  }

  onMount(() => {
    maplibregl.setWorkerUrl(mapWorkerUrl);
    try { keyInput = localStorage.getItem(KEY_STORAGE) || import.meta.env.VITE_MAPTILER_KEY || ''; } catch {}
    map = new maplibregl.Map({ container, style: OFFLINE_STYLE, center: [0, 24], zoom: 1.7, minZoom: 1, maxZoom: 6, attributionControl: false });
    map.scrollZoom.setZoomRate(1 / 80);
    map.scrollZoom.setWheelZoomRate(1 / 300);
    map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-left');
    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), 'bottom-right');
    map.on('style.load', () => {
      for (const marker of markers.values()) marker.remove();
      markers.clear();
      for (const marker of clusterMarkers.values()) marker.remove();
      clusterMarkers.clear();
      groupedPoints = makeGroups(located);
      installLayers();
      if (!mapReady) { mapReady = true; setTimeout(() => fitItems(located), 50); }
    });
    map.on('move', queueViewportUpdate);
    map.on('moveend', updateViewport);
    map.on('render', queueMarkerUpdate);
    map.on('idle', renderMarkers);
    resizeObserver = new ResizeObserver(() => map?.resize());
    resizeObserver.observe(container);
    if (keyInput && localStorage.getItem(DETAIL_STORAGE) === 'true') { enableDetail(); }
  });

  onDestroy(() => {
    if (viewportFrame) cancelAnimationFrame(viewportFrame);
    if (markerFrame) cancelAnimationFrame(markerFrame);
    resizeObserver?.disconnect();
    for (const marker of markers.values()) marker.remove();
    markers.clear();
    for (const marker of clusterMarkers.values()) marker.remove();
    clusterMarkers.clear();
    map?.remove();
    map = null;
  });
</script>

<div class="memories-map-shell" class:panel-collapsed={!panelOpen}>
  <div class="map-canvas" bind:this={container} aria-label="Interactive map of geotagged memories"></div>

  <div class="map-topline">
    <span><MapPin size={14} /> {located.length} mapped</span>
    {#if unmappedCount > 0}<span class="muted">{unmappedCount} without location</span>{/if}
  </div>

  <div class="map-actions">
    <button type="button" on:click={() => fitItems(located)} disabled={!located.length} title="Fit all filtered memories"><Scan size={17} /> Fit results</button>
    <button type="button" on:click={() => (showTileSettings = !showTileSettings)} title="Map detail settings"><Layers size={17} /> {detailedMap ? 'Detailed map' : 'Offline map'}</button>
  </div>

  {#if showTileSettings}
    <div class="map-tile-settings">
      <button type="button" class="close-button" on:click={() => (showTileSettings = false)} aria-label="Close map settings"><X size={16} /></button>
      <h3>Map detail</h3>
      <p>The included world map works offline. A MapTiler key adds streets and place labels. Map tiles reveal the viewed area and your IP address to MapTiler; your photos and pin coordinates stay on this device.</p>
      <label for="maptiler-key">MapTiler API key</label>
      <input id="maptiler-key" type="password" bind:value={keyInput} placeholder="Paste your key" autocomplete="off" />
      {#if tileError}<p class="tile-error" role="alert">{tileError}</p>{/if}
      <div class="settings-buttons">
        <button type="button" on:click={enableDetail}>Use detailed map</button>
        <button type="button" on:click={useOfflineMap}>Use offline map</button>
      </div>
      <a href="https://cloud.maptiler.com/account/keys/" target="_blank" rel="noopener noreferrer">Get or manage a MapTiler key</a>
    </div>
  {/if}

  {#if !detailedMap}<div class="offline-label">Offline world overview · Natural Earth</div>{/if}
  {#if located.length >= 2}
    <div class="density-legend" class:with-offline-label={!detailedMap} aria-label="Memory density: blue means fewer photos, yellow means more photos">
      <strong>Memory density</strong><span class="density-scale" aria-hidden="true"></span><span>Fewer</span><span>More</span>
    </div>
  {/if}

  {#if !located.length}
    <div class="map-empty"><MapPin size={28} /><h3>No mapped memories</h3><p>{unmappedCount ? 'These memories have no GPS coordinates. Try another filter or browse them in Memories and Calendar.' : 'No memories match the current filters.'}</p></div>
  {/if}

  <aside class:collapsed={!panelOpen} class:group-open={panelMode === 'group'} class="map-results">
    <div class="panel-header">
      <div><strong>Explore places</strong><span>{visibleItems.length} in view</span></div>
      <button type="button" on:click={() => (panelOpen = !panelOpen)} aria-label={panelOpen ? 'Collapse results panel' : 'Expand results panel'}><ChevronRight size={18} /></button>
    </div>
    {#if panelOpen}
      <div class="panel-tabs">
        <button type="button" class:active={panelMode === 'nearby'} on:click={() => (panelMode = 'nearby')}>In this area</button>
        <button type="button" class:active={panelMode === 'places'} on:click={() => (panelMode = 'places')}>Places</button>
        {#if selectedGroup}<button type="button" class:active={panelMode === 'group'} on:click={() => (panelMode = 'group')}>Photos ({selectedGroup.items.length})</button>{/if}
      </div>
      <div class="panel-scroll">
        {#if panelMode === 'nearby'}
          {#if visibleItems.length === 0}<p class="panel-empty">Move the map to an area with memories.</p>{/if}
          {#each visibleItems.slice(0, 100) as item (item.memory.id)}
            <button type="button" class="result-row" on:click={() => focusMemory(item)} on:dblclick={() => openFeedAt(item.memory)} title="Show on map; double-click to open memory">
              <span class="row-thumbnail"><MemoryMapThumbnail memory={item.memory} /></span>
              <span><strong>{item.memory.dateFormatted}</strong><small>{item.memory.city || item.memory.locationName || 'Location recorded'}</small></span>
              <MapPin size={15} />
            </button>
          {/each}
          {#if visibleItems.length > 100}<small class="more-count">Showing the newest 100 of {visibleItems.length} in view</small>{/if}
        {:else if panelMode === 'places'}
          <div class="place-breadcrumbs">
            <button type="button" on:click={() => clearPlaceTo('country')}>Countries</button>
            {#if $explorerFilter.selectedCountry !== 'all'}<ChevronRight size={13} /><button type="button" on:click={() => clearPlaceTo('city')}>{$explorerFilter.selectedCountry}</button>{/if}
            {#if $explorerFilter.selectedCity !== 'all'}<ChevronRight size={13} /><button type="button" on:click={() => clearPlaceTo('suburb')}>{$explorerFilter.selectedCity}</button>{/if}
          </div>
          <p class="place-hint">{placeLevel === 'country' ? 'Countries' : placeLevel === 'city' ? 'Cities' : 'Neighbourhoods and suburbs'} · {places.length}</p>
          {#each places as place (place.name)}
            <button type="button" class="place-row" on:click={() => choosePlace(place)}><span class="row-thumbnail"><MemoryMapThumbnail memory={place.memories[0].memory} /></span><span class="place-name">{place.name}</span><strong>{place.count}</strong><ChevronRight size={14} /></button>
          {/each}
        {:else if selectedGroup}
          <div class="selected-head"><strong>{selectedGroup.items.length === 1 ? '1 memory' : `${selectedGroup.items.length} memories`}</strong><button type="button" on:click={() => { selectedGroup = null; panelMode = 'nearby'; }} aria-label="Clear selection"><X size={15} /></button></div>
          <div class="group-gallery">
            {#each selectedGroup.items.slice(0, visibleGroupCount) as item (item.memory.id)}
              <button type="button" class="gallery-card" on:click={() => openFeedAt(item.memory)} aria-label={`Open memory from ${item.memory.dateFormatted}`}>
                <span class="gallery-thumbnail"><MemoryMapThumbnail memory={item.memory} /></span>
                <span><strong>{item.memory.dateFormatted}</strong><small>{item.memory.city || item.memory.locationName || 'Location recorded'}</small></span>
              </button>
            {/each}
          </div>
          {#if selectedGroup.items.length > visibleGroupCount}<button type="button" class="show-more" on:click={() => (visibleGroupCount += 40)}>Show more ({selectedGroup.items.length - visibleGroupCount} remaining)</button>{/if}
        {/if}
      </div>
    {/if}
  </aside>
</div>

<style>
  .memories-map-shell { flex: 1; min-height: 0; position: relative; overflow: hidden; border-radius: 14px; background: #121b2b; color: #f7f9fc; }
  .map-canvas { position: absolute; inset: 0; }
  .map-topline, .map-actions, .offline-label { position: absolute; z-index: 3; }
  .map-topline { left: 16px; top: 16px; display: flex; gap: 10px; align-items: center; padding: 9px 12px; border-radius: 10px; background: #111923e8; font-size: 12px; font-weight: 650; }
  .map-topline span { display: inline-flex; align-items: center; gap: 5px; }
  .muted { color: #b4cad8; font-weight: 500; }
  .map-actions { left: 16px; top: 62px; display: flex; gap: 8px; }
  .map-actions button, .settings-buttons button { border: 0; border-radius: 9px; background: #f7f9fc; color: #162537; padding: 8px 10px; display: inline-flex; gap: 6px; align-items: center; font-size: 12px; font-weight: 700; cursor: pointer; }
  .map-actions button:disabled { opacity: .55; cursor: not-allowed; }
  .map-actions button:hover:not(:disabled), .settings-buttons button:hover { background: #d9efff; }
  .offline-label { bottom: 10px; left: 16px; color: #c2d4df; font-size: 10px; background: #111923dc; border-radius: 6px; padding: 5px 7px; }
  .density-legend { position: absolute; z-index: 3; left: 16px; bottom: 12px; display: grid; grid-template-columns: 1fr 1fr; column-gap: 9px; align-items: center; width: 148px; padding: 8px 10px; border-radius: 8px; background: #111923e8; color: #cbdde8; font-size: 9px; }
  .density-legend.with-offline-label { bottom: 42px; }
  .density-legend strong { grid-column: 1 / -1; color: #f2f8fc; font-size: 10px; margin-bottom: 6px; }
  .density-scale { grid-column: 1 / -1; height: 7px; border-radius: 99px; background: linear-gradient(90deg, #167db4, #20b5d4, #77d9df, #f4dd6d, #fff2b0); margin-bottom: 4px; }
  .density-legend span:last-child { text-align: right; }
  .map-results { position: absolute; z-index: 3; right: 16px; top: 16px; bottom: 16px; width: 420px; display: flex; flex-direction: column; background: #101923f2; border: 1px solid #57718888; border-radius: 14px; overflow: hidden; }
  .map-results.group-open { width: 440px; }
  .map-results.collapsed { bottom: auto; width: 190px; }
  .panel-header { display: flex; align-items: center; justify-content: space-between; padding: 14px; }
  .panel-header div { display: flex; flex-direction: column; gap: 3px; }
  .panel-header strong { font-size: 14px; }
  .panel-header span { color: #acc4d1; font-size: 11px; }
  .panel-header button, .selected-head button, .close-button { color: #d6e6ef; background: transparent; border: 0; cursor: pointer; padding: 4px; }
  .collapsed .panel-header button { transform: rotate(180deg); }
  .panel-tabs { display: flex; padding: 0 12px 9px; gap: 5px; }
  .panel-tabs button { flex: 1; border: 0; border-radius: 7px; padding: 7px; background: #243749; color: #c0d4e0; font-size: 11px; cursor: pointer; }
  .panel-tabs button.active { background: #e7f4ff; color: #11283b; font-weight: 750; }
  .panel-scroll { min-height: 0; overflow: auto; padding: 0 8px 10px; }
  .result-row, .place-row { width: 100%; display: flex; align-items: center; gap: 9px; border: 0; border-radius: 9px; padding: 7px; background: transparent; color: #f1f7fb; text-align: left; cursor: pointer; }
  .result-row:hover, .place-row:hover { background: #29475c; }
  .row-thumbnail { display: block; width: 108px; aspect-ratio: 3 / 4; flex: none; overflow: hidden; border-radius: 8px; background: #294b63; --map-thumbnail-pip-border: 2.5px; --map-thumbnail-pip-radius: 9px; }
  .result-row > span:not(.row-thumbnail) { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 3px; }
  .result-row strong { font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .result-row small { font-size: 11px; color: #b9cedb; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .place-row { justify-content: space-between; padding: 11px 9px; font-size: 12px; }
  .place-row strong { color: #a6dfff; }
  .place-row .place-name { min-width: 0; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .place-breadcrumbs { display: flex; align-items: center; gap: 2px; flex-wrap: wrap; padding: 4px 8px 0; color: #9ecce5; }
  .place-breadcrumbs button { border: 0; background: transparent; color: #9edaff; cursor: pointer; padding: 5px 2px; font-size: 11px; }
  .place-hint { margin: 5px 9px; color: #a9becc; font-size: 11px; }
  .selected-head { display: flex; align-items: center; justify-content: space-between; padding: 7px 9px; font-size: 11px; color: #a6dfff; }
  .group-gallery { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; padding: 4px 7px; }
  .gallery-card { display: flex; flex-direction: column; overflow: hidden; padding: 0; border: 1px solid #3d5e72; border-radius: 9px; color: #eef8ff; background: #1b3041; text-align: left; cursor: pointer; }
  .gallery-card:hover { border-color: #9edaff; background: #25465c; }
  .gallery-thumbnail { display: block; width: 100%; aspect-ratio: 3 / 4; overflow: hidden; background: #294b63; }
  .gallery-card > span:last-child { display: flex; flex-direction: column; gap: 3px; padding: 7px; min-width: 0; }
  .gallery-card strong, .gallery-card small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .gallery-card strong { font-size: 11px; }
  .gallery-card small { font-size: 10px; color: #b9cedb; }
  .show-more { display: block; width: calc(100% - 14px); margin: 8px 7px; border: 1px solid #638ca5; border-radius: 8px; padding: 9px; background: #1e3b50; color: #d9f1ff; font-size: 11px; cursor: pointer; }
  .more-count, .panel-empty { display: block; padding: 12px; font-size: 11px; color: #bed1dd; }
  .map-tile-settings { position: absolute; z-index: 5; top: 106px; left: 16px; width: min(320px, calc(100% - 34px)); padding: 16px; border-radius: 12px; background: #101923; border: 1px solid #69889d; box-shadow: 0 12px 28px #0009; }
  .map-tile-settings h3 { font-size: 15px; margin: 0 0 8px; }
  .map-tile-settings p { font-size: 11px; line-height: 1.5; color: #bcd0dc; margin: 0 0 10px; }
  .map-tile-settings label { display: block; font-size: 11px; margin-bottom: 5px; }
  .map-tile-settings input { width: 100%; border: 1px solid #6c889a; background: #162a3b; border-radius: 7px; color: white; padding: 8px; }
  .map-tile-settings .tile-error { color: #ffbdbd; margin-top: 8px; }
  .settings-buttons { display: flex; gap: 7px; margin: 11px 0; }
  .settings-buttons button { font-size: 10px; }
  .map-tile-settings a { color: #9cdcff; font-size: 11px; }
  .close-button { position: absolute; right: 9px; top: 9px; }
  .map-empty { position: absolute; z-index: 2; left: 50%; top: 50%; transform: translate(-50%, -50%); width: min(300px, 60%); text-align: center; padding: 25px; border-radius: 12px; background: #101923ed; }
  .map-empty h3 { margin: 8px 0; font-size: 16px; }
  .map-empty p { margin: 0; color: #bed1dd; font-size: 12px; line-height: 1.5; }
  :global(.memory-map-pin) { display: block; position: absolute; box-sizing: border-box; width: 96px; height: 128px; padding: 0; border: 3px solid white; border-radius: 12px; background: #22435b; box-shadow: 0 4px 14px #06111ec0; cursor: pointer; }
  :global(.memory-map-pin::after) { content: ''; position: absolute; left: calc(50% - 10px); bottom: -13px; width: 20px; height: 13px; background: white; clip-path: polygon(0 0, 100% 0, 50% 100%); }
  :global(.memory-map-pin-photo) { position: absolute; inset: 0; overflow: hidden; border-radius: 8px; }
  :global(.memory-map-pin-main) { display: block; width: 100%; height: 100%; object-fit: cover; }
  :global(.memory-map-pin-pip) { position: absolute; left: 4%; top: 4%; width: 40%; aspect-ratio: 3 / 4; border: 2px solid white; border-radius: 6px; object-fit: cover; box-shadow: 0 2px 7px #06111eb8; }
  :global(.memory-map-pin-count) { position: absolute; z-index: 1; right: -11px; top: -12px; min-width: 28px; height: 28px; display: grid; place-items: center; border: 2px solid #fff; border-radius: 99px; background: #ffe66b; color: #13293b; font-size: 11px; font-weight: 850; padding: 0 6px; box-shadow: 0 2px 8px #06111eb0; }
  :global(.memory-map-cluster) { width: var(--cluster-size, 42px); height: var(--cluster-size, 42px); display: grid; place-items: center; padding: 0; border: 3px solid #c2efff; border-radius: 50%; background: #f5f9ff; color: #112338; font-size: 13px; font-weight: 850; font-variant-numeric: tabular-nums; cursor: pointer; box-shadow: 0 4px 14px #06111ec9, 0 0 0 5px #d8f4ff45; }
  :global(.memory-map-pin:focus-visible), button:focus-visible, input:focus-visible, a:focus-visible { outline: 2px solid #87d7ff; outline-offset: 3px; }
  :global(.maplibregl-control-container) { color: #173247; }
  .memories-map-shell :global(.maplibregl-ctrl-bottom-right) { right: 458px; bottom: 10px; }
  .memories-map-shell.panel-collapsed :global(.maplibregl-ctrl-bottom-right) { right: 206px; }
  @media (max-width: 1100px) {
    .map-results { width: 360px; }
    .map-results.group-open { width: 380px; }
    .row-thumbnail { width: 92px; }
    .memories-map-shell :global(.maplibregl-ctrl-bottom-right) { right: 398px; }
  }
</style>
