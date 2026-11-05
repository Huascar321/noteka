<script lang="ts">
  import { weatherKind } from './weather';
  let { code, night = false, size = 32, active = true }: { code: number; night?: boolean | null; size?: number; active?: boolean } = $props();
  const kind = $derived(weatherKind(code));
  const celestial = $derived(kind === 'clear' || kind === 'mainly-clear' || kind === 'partly-cloudy');
  const cloud = $derived(kind !== 'clear' && kind !== 'unknown');
  const wet = $derived(['drizzle', 'freezing-drizzle', 'rain', 'freezing-rain', 'showers', 'thunderstorm', 'hailstorm'].includes(kind));
  const snow = $derived(['snow', 'snow-grains', 'snow-showers', 'freezing-drizzle', 'freezing-rain', 'hailstorm'].includes(kind));
  const fog = $derived(kind === 'fog' || kind === 'rime-fog');
  const storm = $derived(kind === 'thunderstorm' || kind === 'hailstorm');
  const intense = $derived([55,57,65,67,75,82,86,97,99].includes(code));
</script>

<svg class="weather-icon" class:weather-icon-paused={!active} data-weather-kind={kind} data-intense={intense} width={size} height={size} viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
  {#if celestial}
    {#if night === null}<circle cx="24" cy="17" r="7" />
    {:else if night}<g class="weather-celestial"><path d="M27 5a10 10 0 1 0 10 15A9 9 0 0 1 27 5Z" /><path d="M13 8v3m-1.5-1.5h3" /></g>
    {:else}<g class="weather-celestial"><circle cx="24" cy="17" r="7" /><path d="M24 5v3m0 18v3M12 17h3m18 0h3M15.5 8.5l2 2m13 13 2 2m0-17-2 2m-13 13-2 2" /></g>{/if}
  {/if}
  {#if cloud}
    {#if kind === 'overcast'}<path class="weather-cloud-back" d="M6 24a5 5 0 0 1 4-9 7 7 0 0 1 13-3 6 6 0 0 1 8 5" />{/if}
    <path class="weather-cloud" fill="var(--weather-icon-backdrop,var(--surface))" d="M11 30a6 6 0 0 1 1-12 9 9 0 0 1 17-1 7 7 0 1 1 3 13H11Z" />
  {/if}
  {#if fog}<g class="weather-fog"><path d="M12 35h22M16 40h18" />{#if kind === 'rime-fog'}<circle cx="11" cy="40" r=".9" fill="currentColor" stroke="none" />{/if}</g>{/if}
  {#if wet}<g class="weather-rain"><path d="m15 34-2 4m9-4-2 4" />{#if kind !== 'drizzle' && kind !== 'freezing-drizzle'}<path d="m29 34-2 4" />{/if}{#if intense}<path d="m34 34-2 4m-15 2-1 2m9-2-1 2" />{/if}</g>{/if}
  {#if snow}<g class="weather-snow"><circle cx="16" cy="37" r="1.3" fill="currentColor" stroke="none" /><circle cx="25" cy="40" r="1.3" fill="currentColor" stroke="none" /><circle cx="33" cy="36" r="1.3" fill="currentColor" stroke="none" />{#if intense}<circle cx="10" cy="41" r="1.3" fill="currentColor" stroke="none" />{/if}</g>{/if}
  {#if storm}<path class="weather-lightning" d="m25 31-3 6h4l-3 6" />{/if}
  {#if kind === 'unknown'}<circle cx="24" cy="23" r="12" /><path d="M20 19a4 4 0 1 1 5 4c-1 .5-1 1-1 2m0 4h.01" />{/if}
</svg>
