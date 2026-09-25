import type { ExplorerMemory } from './types';

export interface LocatedMemory {
  memory: ExplorerMemory;
  latitude: number;
  longitude: number;
}

export interface MemoryPlace {
  name: string;
  count: number;
  memories: LocatedMemory[];
  value: string | null;
}

export type PlaceLevel = 'country' | 'city' | 'suburb';

export function getLocatedMemories(memories: ExplorerMemory[]): LocatedMemory[] {
  return memories.flatMap((memory) => {
    const latitude = memory.location?.latitude;
    const longitude = memory.location?.longitude;
    return typeof latitude === 'number' && typeof longitude === 'number' &&
      Number.isFinite(latitude) && Number.isFinite(longitude) &&
      latitude >= -90 && latitude <= 90 && longitude >= -180 && longitude <= 180
      ? [{ memory, latitude, longitude }]
      : [];
  });
}

export function getMemoryPlaces(memories: LocatedMemory[], level: PlaceLevel = 'city'): MemoryPlace[] {
  const groups = new Map<string, LocatedMemory[]>();
  for (const item of memories) {
    const value = item.memory[level]?.trim() || null;
    const group = groups.get(value ?? '') ?? [];
    group.push(item);
    groups.set(value ?? '', group);
  }
  const missingLabel = level === 'country' ? 'Unknown country' : level === 'city' ? 'Unknown city' : 'No suburb name';
  return [...groups.entries()]
    .map(([value, items]) => ({ name: value || missingLabel, value: value || null, count: items.length, memories: items }))
    .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
}

// Normalise wrapped longitudes so bounds crossing the antimeridian still include points.
export function isInsideMapBounds(
  item: LocatedMemory,
  west: number,
  south: number,
  east: number,
  north: number,
): boolean {
  if (item.latitude < south || item.latitude > north) return false;
  if (east - west >= 360) return true;
  const longitude = ((item.longitude - west) % 360 + 360) % 360 + west;
  return longitude <= east;
}

export function getMemoryCover(memory: ExplorerMemory): string | undefined {
  const isImage = (path?: string) => path && !/\.(mp4|mov|m4v|webm)(\?|$)/i.test(path);
  return (isImage(memory.primaryPath) && memory.primaryPath) ||
    (isImage(memory.secondaryPath) && memory.secondaryPath) || undefined;
}
