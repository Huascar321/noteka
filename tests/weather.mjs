import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readCity, readForecast, readWeatherMode, saveCity, saveForecast, saveWeatherMode, clearWeather, fetchForecast, weatherDescription, weatherKind } from '../src/lib/weather.ts';
const data = new Map();
globalThis.localStorage = { getItem: key => data.get(key) ?? null, setItem: (key, value) => data.set(key, value), removeItem: key => data.delete(key) };
const city = { id: 1, name: 'Trinidad', admin1: 'Beni', country: 'Bolivia', latitude: -14.83, longitude: -64.9 };
const days = Array.from({ length: 8 }, (_, i) => `2026-10-${String(26 + i).padStart(2, '0')}`);
test('weather opt-in, cache isolation, invalid storage and removal', () => {
  data.clear(); assert.equal(readCity(), null); saveCity(city); assert.deepEqual(readCity(), city);
  assert.equal(readWeatherMode(), 'fixed'); saveWeatherMode('current'); assert.equal(readWeatherMode(), 'current');
  const forecast = { city, updated: 100, timezone: 'America/La_Paz', current: { temperature: 27, code: 3 }, days: days.map(date => ({ date, code: 3, min: 20, max: 30 })) };
  saveForecast(forecast); assert.deepEqual(readForecast(city), forecast); assert.equal(readForecast({ ...city, id: 2 }), null);
  data.set('noteka.weather.cache.v1', '{bad'); assert.equal(readForecast(city), null);
  clearWeather(); assert.equal(readCity(), null); assert.equal(data.size, 0);
});
test('forecast requests only required data and validates responses', async () => {
  const urls = [];
  globalThis.fetch = async (url, options) => { urls.push(new URL(url)); assert.equal(options.credentials, 'omit'); return { ok: true, json: async () => ({ timezone: 'America/La_Paz', current: { temperature_2m: 28, weather_code: 61, is_day: 0 }, daily: { time: days, weather_code: days.map(() => 61), temperature_2m_max: days.map(() => 31), temperature_2m_min: days.map(() => 22) } }) }; };
  const forecast = await fetchForecast(city); assert.equal(forecast.days.length, 8); assert.equal(weatherDescription(forecast.current.code), 'Light rain'); assert.equal(forecast.current.isDay, 0);
  assert.equal(urls[0].host, 'api.open-meteo.com'); assert.equal(urls[0].searchParams.get('forecast_days'), '8'); assert.equal(urls[0].searchParams.get('timezone'), 'auto');
  assert.match(urls[0].searchParams.get('current'), /is_day/);
  globalThis.fetch = async () => ({ ok: true, json: async () => ({ daily: {} }) });
  await assert.rejects(fetchForecast(city), /Invalid forecast/);
});
test('WMO conditions distinguish precipitation, fog and unknown codes', () => {
  for (const [code, kind] of [[0,'clear'],[1,'mainly-clear'],[2,'partly-cloudy'],[3,'overcast'],[45,'fog'],[48,'rime-fog'],[51,'drizzle'],[57,'freezing-drizzle'],[65,'rain'],[67,'freezing-rain'],[75,'snow'],[77,'snow-grains'],[82,'showers'],[86,'snow-showers'],[95,'thunderstorm'],[99,'hailstorm'],[42,'unknown']]) assert.equal(weatherKind(code), kind);
  assert.equal(weatherDescription(42), 'Weather unavailable');
});
