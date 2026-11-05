<script lang="ts">
  import { onMount } from 'svelte';
  import AppIcon from './AppIcon.svelte';
  import WeatherIcon from './WeatherIcon.svelte';
  import { currentWeatherLocation, openExternal } from './api';
  import { desktopPlatform } from './window-chrome';
  import { cityLabel, clearWeather, fetchForecast, readCity, readForecast, readWeatherMode, saveCity, saveForecast, saveWeatherMode, validCity, weatherDescription, type City, type Forecast } from './weather';
  let { visible = true }: { visible?: boolean } = $props();
  let weatherRoot = $state<HTMLElement>();
  let onScreen = $state(false);
  let pageVisible = $state(true);
  const animate = $derived(visible && onScreen && pageVisible);
  let city = $state<City | null>(null);
  let forecast = $state<Forecast | null>(null);
  let loading = $state(false);
  let error = $state('');
  let storageError = $state('');
  let locating = $state(false);
  let forecastController: AbortController | undefined;
  let locationGeneration = 0;
  const platform = desktopPlatform();
  const linux = platform === 'other' && /Linux/i.test(navigator.platform || navigator.userAgent);
  function disable() {
    forecastController?.abort(); locationGeneration++; locating = false;
    try { clearWeather(); } catch { storageError = 'Could not remove saved weather on this device.'; return; }
    city = null; forecast = null; loading = false; error = ''; storageError = '';
  }
  async function locate() {
    const request = ++locationGeneration;
    forecastController?.abort();
    locating = true; loading = true; error = '';
    try {
      const raw = await currentWeatherLocation();
      if (request !== locationGeneration) return;
      const candidate = { ...(raw as object), id: -1 };
      if (!validCity(candidate)) throw new Error('Location unavailable. Check Location Services and try again.');
      const controller = forecastController = new AbortController();
      const next = await fetchForecast(candidate, controller.signal);
      if (request !== locationGeneration) return;
      try { saveCity(candidate); saveForecast(next); saveWeatherMode('current'); }
      catch { storageError = 'Could not save weather on this device.'; return; }
      city = candidate; forecast = next; storageError = '';
    } catch (cause) {
      if (request === locationGeneration) error = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Location unavailable. Check Location Services and try again.';
    } finally { if (request === locationGeneration) { locating = false; loading = false; } }
  }
  async function openAttribution(event: MouseEvent) {
    event.preventDefault();
    try { await openExternal('https://open-meteo.com/'); }
    catch { error = 'Could not open the weather provider in your browser.'; }
  }
  function dayName(date: string) {
    const today = forecast ? new Intl.DateTimeFormat('en-CA', { timeZone: forecast.timezone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date()) : '';
    return date === today ? 'Today' : new Intl.DateTimeFormat('en', { weekday: 'short', timeZone: 'UTC' }).format(new Date(`${date}T12:00:00Z`));
  }
  function scrollForecast(event: KeyboardEvent) {
    const region = event.currentTarget as HTMLElement;
    if (region.scrollWidth <= region.clientWidth) return;
    if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
      event.preventDefault();
      region.scrollBy({ left: event.key === 'ArrowRight' ? 76 : -76, behavior: 'auto' });
    } else if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault();
      region.scrollTo({ left: event.key === 'Home' ? 0 : region.scrollWidth, behavior: 'auto' });
    }
  }
  onMount(() => {
    const visibility = () => { pageVisible = document.visibilityState === 'visible'; };
    visibility(); document.addEventListener('visibilitychange', visibility);
    const observer = new IntersectionObserver(entries => { onScreen = entries.some(entry => entry.isIntersecting); });
    if (weatherRoot) observer.observe(weatherRoot);
    if (platform === 'macos' && readWeatherMode() === 'current') city = readCity();
    if (city) { forecast = readForecast(city); void locate(); }
    const timer = window.setInterval(() => { if (city && !locating) void locate(); }, 60 * 60 * 1000);
    return () => { clearInterval(timer); forecastController?.abort(); locationGeneration++; observer.disconnect(); document.removeEventListener('visibilitychange', visibility); };
  });
</script>

<section bind:this={weatherRoot} class="home-weather" aria-label="Weather">
  {#if !city}
    <div class="weather-empty"><AppIcon name="cloud" size={30} /><div><strong>Weather</strong>{#if platform === 'macos'}<p>Use your current location for today's weather and the next seven days. macOS asks for permission; Apple may receive coordinates to identify the city and Open-Meteo receives them for the forecast.</p>{:else if platform === 'windows'}<p>Automatic location for weather is not yet available on Windows.</p>{:else if linux}<p>Automatic location for weather is not yet available on Linux.</p>{:else if platform === 'other'}<p>Automatic location for weather is not yet available on this system.</p>{:else}<p>Automatic location for weather requires the desktop app.</p>{/if}</div>{#if platform === 'macos'}<button disabled={locating} onclick={() => void locate()}>{locating ? 'Finding location…' : 'Set location'}</button>{/if}</div>
  {:else}
    <div class="weather-layout">
      <div class="weather-current">
        <div class="weather-place"><strong>{cityLabel(city)}</strong><span>Weather</span></div>
        <WeatherIcon code={forecast?.current.code ?? -1} night={forecast?.current.isDay === undefined ? null : forecast.current.isDay === 0} size={40} active={animate} />
        <div class="weather-now"><strong>{forecast ? `${Math.round(forecast.current.temperature)} °C` : 'Weather'}</strong>{#if forecast}<span>{weatherDescription(forecast.current.code)}</span>{/if}</div>
      </div>
      {#if forecast}<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (Keyboard access to the scrollable forecast.) --><div class="weather-days" role="region" aria-label="Eight-day forecast" tabindex="0" onkeydown={scrollForecast}>{#each forecast.days as day (day.date)}<div class="weather-day"><time datetime={day.date} title={day.date}>{dayName(day.date)}<span class="sr-only">, {day.date}</span></time><WeatherIcon code={day.code} size={27} active={animate} /><span class="sr-only">{weatherDescription(day.code)}</span><div class="weather-temperatures"><strong><span class="sr-only">High </span>{Math.round(day.max)}°</strong><span><span class="sr-only">Low </span>{Math.round(day.min)}°</span></div></div>{/each}</div>{/if}
    </div>
    <div class="weather-footer"><div class="weather-meta">{#if loading}Updating…{:else if forecast}Updated {new Date(forecast.updated).toLocaleString()}{/if} <a href="https://open-meteo.com/" aria-label="Weather data by Open-Meteo" onclick={openAttribution}>Open-Meteo</a></div><div class="weather-actions"><button disabled={locating} onclick={() => void locate()}>Update location</button><button onclick={disable}>Disable</button></div></div>
  {/if}
  {#if error}<p role="status" class="weather-error">{error} {#if city}<button onclick={() => void locate()}>Retry</button>{/if}</p>{/if}
  {#if storageError}<p role="status" class="weather-error">{storageError}</p>{/if}
</section>
