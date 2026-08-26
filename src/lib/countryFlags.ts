// Mapping country names and ISO codes to 2-letter alpha-2 codes for universal flag images & vector assets

const COUNTRY_TO_ISO_MAP: Record<string, string> = {
  'united kingdom': 'gb',
  'uk': 'gb',
  'great britain': 'gb',
  'england': 'gb',
  'scotland': 'gb-sct',
  'wales': 'gb-wls',
  'northern ireland': 'gb-nir',
  'united states': 'us',
  'united states of america': 'us',
  'usa': 'us',
  'us': 'us',
  'france': 'fr',
  'germany': 'de',
  'deutschland': 'de',
  'spain': 'es',
  'españa': 'es',
  'italy': 'it',
  'italia': 'it',
  'canada': 'ca',
  'australia': 'au',
  'netherlands': 'nl',
  'holland': 'nl',
  'switzerland': 'ch',
  'belgium': 'be',
  'austria': 'at',
  'portugal': 'pt',
  'sweden': 'se',
  'norway': 'no',
  'denmark': 'dk',
  'finland': 'fi',
  'ireland': 'ie',
  'japan': 'jp',
  'south korea': 'kr',
  'korea': 'kr',
  'china': 'cn',
  'singapore': 'sg',
  'brazil': 'br',
  'brasil': 'br',
  'mexico': 'mx',
  'méxico': 'mx',
  'argentina': 'ar',
  'greece': 'gr',
  'turkey': 'tr',
  'türkiye': 'tr',
  'united arab emirates': 'ae',
  'uae': 'ae',
  'dubai': 'ae',
  'india': 'in',
  'thailand': 'th',
  'vietnam': 'vn',
  'indonesia': 'id',
  'malaysia': 'my',
  'philippines': 'ph',
  'new zealand': 'nz',
  'south africa': 'za',
  'egypt': 'eg',
  'morocco': 'ma',
  'croatia': 'hr',
  'czech republic': 'cz',
  'czechia': 'cz',
  'poland': 'pl',
  'hungary': 'hu',
  'romania': 'ro',
  'bulgaria': 'bg',
  'iceland': 'is',
  'colombia': 'co',
  'chile': 'cl',
  'peru': 'pe',
  'perú': 'pe',
  'hong kong': 'hk',
  'taiwan': 'tw',
  'israel': 'il',
  'saudi arabia': 'sa',
  'qatar': 'qa',
  'kuwait': 'kw',
  'ukraine': 'ua',
  'serbia': 'rs',
  'slovakia': 'sk',
  'slovenia': 'si',
  'estonia': 'ee',
  'latvia': 'lv',
  'lithuania': 'lt',
  'luxembourg': 'lu',
  'monaco': 'mc',
  'malta': 'mt',
  'cyprus': 'cy',
  'new caledonia': 'nc',
  'puerto rico': 'pr',
  'costa rica': 'cr',
  'panama': 'pa',
  'panamá': 'pa',
  'uruguay': 'uy',
  'ecuador': 'ec',
};

export function getCountryIsoCode(countryNameOrCode?: string): string {
  if (!countryNameOrCode) return '';
  const clean = countryNameOrCode.toLowerCase().trim();
  
  if (COUNTRY_TO_ISO_MAP[clean]) {
    return COUNTRY_TO_ISO_MAP[clean];
  }
  
  if (clean.length === 2 && /^[a-z]{2}$/.test(clean)) {
    return clean;
  }

  return '';
}

export function getCountryFlagImgUrl(countryNameOrCode?: string): string {
  const code = getCountryIsoCode(countryNameOrCode);
  if (!code) return '';
  // High-res crisp vector flag PNG (FlagCDN)
  return `https://flagcdn.com/w40/${code.toLowerCase()}.png`;
}
