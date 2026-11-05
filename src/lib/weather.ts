export type City = { id: number; name: string; admin1?: string; country?: string; latitude: number; longitude: number };
export type Forecast = { city: City; updated: number; timezone: string; current: { temperature: number; code: number; isDay?: 0 | 1 }; days: { date: string; code: number; max: number; min: number }[] };
const CITY_KEY = 'noteka.weather.city.v1';
const CACHE_KEY = 'noteka.weather.cache.v1';
const MODE_KEY = 'noteka.weather.mode.v1';
const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);
export function validCity(value: unknown): value is City {
  if (!value || typeof value !== 'object') return false;
  const c = value as Record<string, unknown>;
  return finite(c.id) && typeof c.name === 'string' && c.name.length > 0 && finite(c.latitude) && Math.abs(c.latitude) <= 90 && finite(c.longitude) && Math.abs(c.longitude) <= 180 && (c.admin1 === undefined || typeof c.admin1 === 'string') && (c.country === undefined || typeof c.country === 'string');
}
export function cityLabel(city: City) { return [city.name, city.admin1, city.country].filter(Boolean).join(', '); }
export function validForecast(value: unknown): value is Forecast {
  if (!value || typeof value !== 'object') return false;
  const f = value as Forecast;
  try { new Intl.DateTimeFormat('en', { timeZone: f.timezone }); } catch { return false; }
  return validCity(f.city) && finite(f.updated) && typeof f.timezone === 'string' && !!f.current && finite(f.current.temperature) && finite(f.current.code) && (f.current.isDay === undefined || f.current.isDay === 0 || f.current.isDay === 1) && Array.isArray(f.days) && f.days.length === 8 && f.days.every(d => d && /^\d{4}-\d{2}-\d{2}$/.test(d.date) && finite(d.code) && finite(d.max) && finite(d.min));
}
function read(key: string): unknown { try { const raw = localStorage.getItem(key); return raw ? JSON.parse(raw) : null; } catch { return null; } }
export function readCity(): City | null { const city = read(CITY_KEY); return validCity(city) ? city : null; }
export function readWeatherMode(): 'fixed' | 'current' { return read(MODE_KEY) === 'current' ? 'current' : 'fixed'; }
export function saveWeatherMode(mode: 'fixed' | 'current') { localStorage.setItem(MODE_KEY, JSON.stringify(mode)); }
export function readForecast(city: City): Forecast | null { const f = read(CACHE_KEY); return validForecast(f) && f.city.id === city.id && f.city.latitude === city.latitude && f.city.longitude === city.longitude ? f : null; }
export function saveCity(city: City) { localStorage.setItem(CITY_KEY, JSON.stringify(city)); localStorage.removeItem(CACHE_KEY); }
export function saveForecast(forecast: Forecast) { localStorage.setItem(CACHE_KEY, JSON.stringify(forecast)); }
export function clearWeather() { localStorage.removeItem(MODE_KEY); localStorage.removeItem(CITY_KEY); localStorage.removeItem(CACHE_KEY); }
async function json(url: URL, signal?: AbortSignal): Promise<any> {
  const response = await fetch(url, { signal, credentials: 'omit', referrerPolicy: 'no-referrer' });
  if (!response.ok) throw new Error('Weather service unavailable');
  return response.json();
}
export async function fetchForecast(city: City, signal?: AbortSignal): Promise<Forecast> {
  const url = new URL('https://api.open-meteo.com/v1/forecast');
  url.search = new URLSearchParams({ latitude: String(city.latitude), longitude: String(city.longitude), current: 'temperature_2m,weather_code,is_day', daily: 'weather_code,temperature_2m_max,temperature_2m_min', timezone: 'auto', forecast_days: '8' }).toString();
  const data = await json(url, signal);
  if (!Array.isArray(data.daily?.time) || !Array.isArray(data.daily?.weather_code) || !Array.isArray(data.daily?.temperature_2m_max) || !Array.isArray(data.daily?.temperature_2m_min)) throw new Error('Invalid forecast');
  const result = { city, updated: Date.now(), timezone: data.timezone, current: { temperature: data.current?.temperature_2m, code: data.current?.weather_code, isDay: data.current?.is_day }, days: data.daily.time.map((date: string, i: number) => ({ date, code: data.daily.weather_code[i], max: data.daily.temperature_2m_max[i], min: data.daily.temperature_2m_min[i] })) };
  if (!validForecast(result)) throw new Error('Invalid forecast');
  return result;
}
export type WeatherKind = 'clear' | 'mainly-clear' | 'partly-cloudy' | 'overcast' | 'fog' | 'rime-fog' | 'drizzle' | 'freezing-drizzle' | 'rain' | 'freezing-rain' | 'snow' | 'snow-grains' | 'showers' | 'snow-showers' | 'thunderstorm' | 'hailstorm' | 'unknown';
const conditions: Record<number, [WeatherKind, string]> = {
  0: ['clear', 'Clear'], 1: ['mainly-clear', 'Mainly clear'], 2: ['partly-cloudy', 'Partly cloudy'], 3: ['overcast', 'Overcast'],
  45: ['fog', 'Fog'], 48: ['rime-fog', 'Rime fog'],
  51: ['drizzle', 'Light drizzle'], 53: ['drizzle', 'Moderate drizzle'], 55: ['drizzle', 'Dense drizzle'],
  56: ['freezing-drizzle', 'Freezing drizzle'], 57: ['freezing-drizzle', 'Dense freezing drizzle'],
  61: ['rain', 'Light rain'], 63: ['rain', 'Moderate rain'], 65: ['rain', 'Heavy rain'],
  66: ['freezing-rain', 'Freezing rain'], 67: ['freezing-rain', 'Heavy freezing rain'],
  71: ['snow', 'Light snow'], 73: ['snow', 'Moderate snow'], 75: ['snow', 'Heavy snow'], 77: ['snow-grains', 'Snow grains'],
  80: ['showers', 'Light showers'], 81: ['showers', 'Moderate showers'], 82: ['showers', 'Heavy showers'],
  85: ['snow-showers', 'Light snow showers'], 86: ['snow-showers', 'Heavy snow showers'],
  95: ['thunderstorm', 'Thunderstorm'], 96: ['hailstorm', 'Thunderstorm with hail'], 97: ['thunderstorm', 'Heavy thunderstorm'], 99: ['hailstorm', 'Thunderstorm with heavy hail'],
};
export function weatherKind(code: number): WeatherKind { return conditions[code]?.[0] ?? 'unknown'; }
export function weatherDescription(code: number): string { return conditions[code]?.[1] ?? 'Weather unavailable'; }
