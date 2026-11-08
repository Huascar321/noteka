<script lang="ts">
  import mark from './noteka-brand.json';
  let { size = 24 }: { size?: number } = $props();
</script>

<svg class="noteka-mark" data-noteka-mark width={size} height={size} viewBox={mark.viewBox} style:--noteka-blue-dark={mark.blueDark} aria-hidden="true" focusable="false">
  <path class="noteka-mark-body" fill={mark.blue} d={mark.body} />
  <path class="noteka-mark-peel" fill={mark.paper} stroke={mark.paperShade} stroke-width=".75" stroke-linejoin="round" d={mark.peel} />
</svg>

<style>
  .noteka-mark { display: block; flex-shrink: 0; overflow: visible; }
  :global(:root[data-theme='dark']) .noteka-mark-body { fill: var(--noteka-blue-dark); }
  @media (prefers-contrast: more) {
    .noteka-mark-body { fill: var(--text); }
    :global(:root[data-theme='dark']) .noteka-mark-body { fill: var(--text); }
    .noteka-mark-peel { stroke: var(--text); stroke-width: 2; }
  }
</style>
