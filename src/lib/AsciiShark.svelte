<script lang="ts">
  import { onMount } from 'svelte';
  import { AsciiCoverRenderer } from './ascii-cover-renderer';
  import { OceanClock } from './ocean-clock';
  let { active = true }: { active?: boolean } = $props();
  let canvas: HTMLCanvasElement;
  let redraw: (() => void) | undefined;
  $effect(() => { const enabled=active; if(redraw) { void enabled; redraw(); } });
  onMount(() => {
    const renderer=new AsciiCoverRenderer(canvas),clock=new OceanClock();
    const media=matchMedia('(prefers-reduced-motion: reduce)');
    let visible=true,frame=0,disposed=false,width=0,height=0;
    function draw(now:number) {
      frame=0;
      if(disposed||!active||!visible||document.hidden||!width||!height){clock.suspend();return;}
      const time=clock.tick(now,media.matches);
      if(time){renderer.draw(time.seconds,time.delta);canvas.dataset.frame=String(time.frame);}
      if(!media.matches)frame=requestAnimationFrame(draw);
    }
    function wake(){cancelAnimationFrame(frame);frame=0;clock.suspend();if(!disposed&&active&&visible&&!document.hidden&&width&&height)frame=requestAnimationFrame(draw);}
    function measure(){
      const rect=canvas.getBoundingClientRect();width=rect.width;height=rect.height;
      const styles=getComputedStyle(document.documentElement);
      renderer.resize(width,height,devicePixelRatio||1,document.documentElement.dataset.theme!=='light',styles.getPropertyValue('--text').trim(),styles.getPropertyValue('--primary-color').trim());
      // A layout transition must not cancel the next frame or suspend active time.
      if(!frame)wake();
    }
    const resize=new ResizeObserver(measure);resize.observe(canvas);
    const intersection=new IntersectionObserver(entries=>{visible=entries[0]?.isIntersecting??true;wake();});intersection.observe(canvas);
    const appearance=new MutationObserver(measure);appearance.observe(document.documentElement,{attributes:true,attributeFilter:['data-theme','style']});
    media.addEventListener('change',wake);document.addEventListener('visibilitychange',wake);redraw=wake;measure();
    return()=>{disposed=true;cancelAnimationFrame(frame);resize.disconnect();intersection.disconnect();appearance.disconnect();media.removeEventListener('change',wake);document.removeEventListener('visibilitychange',wake);redraw=undefined;};
  });
</script>
<canvas bind:this={canvas} class="ascii-shark" aria-hidden="true"></canvas>
<style>.ascii-shark{display:block;width:100%;height:100%;}</style>
