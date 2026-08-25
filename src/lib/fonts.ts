export interface FontOption {
  id: string;
  name: string;
  category: string;
  cssFont: string;
  cssClass?: string;
}

export const BUILTIN_FONT_OPTIONS: FontOption[] = [
  { id: 'inter', name: 'Inter', category: 'Modern Clean Sans', cssFont: 'font-inter', cssClass: 'font-inter' },
  { id: 'roboto', name: 'Roboto', category: 'BeReal Classic / Punchy', cssFont: 'font-roboto', cssClass: 'font-roboto' },
  { id: 'outfit', name: 'Outfit', category: 'Geometric Aesthetic', cssFont: 'font-outfit', cssClass: 'font-outfit' },
  { id: 'bebas', name: 'Bebas Neue', category: 'Bold Poster / TikTok', cssFont: 'font-bebas', cssClass: 'font-bebas' },
  { id: 'playfair', name: 'Playfair Display', category: 'Editorial Serif', cssFont: 'font-playfair', cssClass: 'font-playfair' },
  { id: 'jetbrains', name: 'JetBrains Mono', category: 'Tech Monospace', cssFont: 'font-jetbrains', cssClass: 'font-jetbrains' },
  { id: 'caveat', name: 'Caveat', category: 'Handwritten Journal', cssFont: 'font-caveat', cssClass: 'font-caveat' },
];
