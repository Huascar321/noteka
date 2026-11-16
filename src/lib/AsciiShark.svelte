<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { AsciiCoverRenderer } from './ascii-cover-renderer';
  import { MAX_OCEAN_FPS, MIN_OCEAN_FPS, OceanClock } from './ocean-clock';
  let { active = true }: { active?: boolean } = $props();
  let canvas: HTMLCanvasElement;
  let redraw: (() => void) | undefined;
  $effect(() => { const enabled=active; if(redraw) { void enabled; redraw(); } });
  onMount(() => {
    const renderer=new AsciiCoverRenderer(canvas),clock=new OceanClock();
    const media=matchMedia('(prefers-reduced-motion: reduce)');
    let visible=true,frame=0,disposed=false,width=0,height=0,publishedAt=-Infinity,resizeTimer=0;
    // Cadencia: máximo mientras la ventana tiene foco y la máquina lo sostiene; el fondo baja a 30 FPS.
    let budget=MAX_OCEAN_FPS,unfocused=false,late=0,lastSample=0,scrolling=0;
    function applyRate(){ clock.setFps(unfocused?MIN_OCEAN_FPS:budget); }
    function sample(now:number) {
      if(unfocused||!lastSample){lastSample=now;return;}
      const interval=1000/budget,slow=now-lastSample>interval*1.35;
      lastSample=now;
      if(!slow){late=0;return;}
      if(++late<10)return;
      budget=Math.max(MIN_OCEAN_FPS,budget/2);late=0;applyRate();
    }
    function draw(now:number) {
      frame=0;
      if(disposed||!active||!visible||document.hidden||!width||!height){clock.suspend();return;}
      sample(now);
      const time=clock.tick(now,media.matches);
      if(time){
        renderer.draw(time.seconds,time.delta);
        // Diagnostics are not presentation: avoid invalidating DOM/style at the render cadence.
        if(now-publishedAt>=250||media.matches){canvas.dataset.frame=String(time.frame);publishedAt=now;}
      }
      if(!media.matches)frame=requestAnimationFrame(draw);
    }
    function wake(){cancelAnimationFrame(frame);frame=0;clock.suspend();lastSample=0;if(!disposed&&active&&visible&&!document.hidden&&width&&height)frame=requestAnimationFrame(draw);}
    function scrollHost(){
      let node:HTMLElement|null=canvas.parentElement;
      while(node){const style=getComputedStyle(node);if((style.overflowY==='auto'||style.overflowY==='scroll')&&node.scrollHeight>node.clientHeight+1)return node;node=node.parentElement;}
      return null;
    }
    const host=scrollHost();
    // Recorrer Home no necesita 120 FPS del renderer: cede el hilo y vuelve al detenerse.
    function onScroll(){if(!scrolling){cancelAnimationFrame(frame);frame=0;clock.suspend();}else clearTimeout(scrolling);scrolling=window.setTimeout(()=>{scrolling=0;wake();},160);}
    function focusChanged(forced?: boolean){unfocused=forced??!document.hasFocus();if(!unfocused){budget=MAX_OCEAN_FPS;late=0;}applyRate();wake();}
    function resizeRenderer(){
      resizeTimer=0;
      const styles=getComputedStyle(document.documentElement);
      renderer.resize(width,height,devicePixelRatio||1,document.documentElement.dataset.theme!=='light',styles.getPropertyValue('--text').trim(),styles.getPropertyValue('--primary-color').trim());
      if(!frame)wake();
    }
    function measure(){
      const initial=!width||!height,rect=canvas.getBoundingClientRect();width=rect.width;height=rect.height;
      if(resizeTimer)clearTimeout(resizeTimer);
      // Width transitions may resize Home every frame. Continue drawing into
      // the existing buffer and rebuild the scene caches once layout settles.
      if(initial||media.matches)resizeRenderer();
      else resizeTimer=window.setTimeout(resizeRenderer,90);
      // A layout transition must not cancel the next frame or suspend active time.
      if(!frame)wake();
    }
    const resize=new ResizeObserver(measure);resize.observe(canvas);
    const intersection=new IntersectionObserver(entries=>{visible=entries[0]?.isIntersecting??true;wake();});intersection.observe(canvas);
    const appearance=new MutationObserver(measure);appearance.observe(document.documentElement,{attributes:true,attributeFilter:['data-theme','style']});
    const onFocus=()=>focusChanged(false),onBlur=()=>focusChanged(true);
    media.addEventListener('change',wake);document.addEventListener('visibilitychange',wake);
    window.addEventListener('focus',onFocus);window.addEventListener('blur',onBlur);
    host?.addEventListener('scroll',onScroll,{passive:true});
    let stopFocus:(()=>void)|undefined;
    if(isTauri())void import('@tauri-apps/api/window').then(async({getCurrentWindow})=>{
      if(disposed)return;
      const off=await getCurrentWindow().onFocusChanged(({payload})=>focusChanged(!payload));
      if(disposed)off();else stopFocus=off;
    }).catch(()=>{});
    redraw=wake;focusChanged();measure();
    return()=>{disposed=true;cancelAnimationFrame(frame);if(scrolling)clearTimeout(scrolling);if(resizeTimer)clearTimeout(resizeTimer);stopFocus?.();resize.disconnect();intersection.disconnect();appearance.disconnect();media.removeEventListener('change',wake);document.removeEventListener('visibilitychange',wake);window.removeEventListener('focus',onFocus);window.removeEventListener('blur',onBlur);host?.removeEventListener('scroll',onScroll);redraw=undefined;};
  });
</script>
<canvas bind:this={canvas} class="ascii-shark" aria-hidden="true"></canvas>
<style>.ascii-shark{display:block;width:100%;height:100%;contain:strict;will-change:transform;}</style>
