import { AsciiOceanRenderer } from './ascii-ocean-renderer';
import { AsciiCreaturesRenderer } from './ascii-creatures';
import { coverSceneAt, type CoverScene } from './ascii-cover-scene';
export class AsciiCoverRenderer {
  private ctx:CanvasRenderingContext2D;
  private canvases:Record<CoverScene,HTMLCanvasElement>={sharks:document.createElement('canvas'),koi:document.createElement('canvas'),jellyfish:document.createElement('canvas')};
  private sharks:AsciiOceanRenderer;private koi:AsciiCreaturesRenderer;private jellyfish:AsciiCreaturesRenderer;
  private width=0;private height=0;private dpr=1;private seconds=0;
  constructor(private canvas:HTMLCanvasElement,private seed=Math.floor(Math.random()*2147483647)){
    const ctx=canvas.getContext('2d');if(!ctx)throw Error('Canvas2D unavailable');this.ctx=ctx;
    this.sharks=new AsciiOceanRenderer(this.canvases.sharks);this.koi=new AsciiCreaturesRenderer(this.canvases.koi,'koi',seed);this.jellyfish=new AsciiCreaturesRenderer(this.canvases.jellyfish,'jellyfish',seed);
  }
  resize(width:number,height:number,dpr:number,dark:boolean,ink:string){
    this.width=Math.max(0,width);this.height=Math.max(0,height);this.dpr=Math.min(2,Math.max(1,dpr));
    this.canvas.width=Math.round(width*this.dpr);this.canvas.height=Math.round(height*this.dpr);
    this.sharks.resize(width,height,dpr,dark,ink);this.koi.resize(width,height,dpr,dark);this.jellyfish.resize(width,height,dpr,dark);this.draw(this.seconds,0);
  }
  draw(seconds:number,delta:number){
    this.seconds=seconds;if(!this.width||!this.height)return;
    const choice=coverSceneAt(this.seed,seconds),ctx=this.ctx;
    const render=(scene:CoverScene,alpha:number)=>{if(scene==='sharks')this.sharks.draw(seconds,delta);else this[scene].draw(seconds);ctx.globalAlpha=alpha;ctx.drawImage(this.canvases[scene],0,0,this.width,this.height);};
    ctx.setTransform(this.dpr,0,0,this.dpr,0,0);ctx.clearRect(0,0,this.width,this.height);render(choice.current,1-choice.blend);if(choice.blend>0)render(choice.next,choice.blend);ctx.globalAlpha=1;
    this.canvas.dataset.scene=choice.current;this.canvas.dataset.sceneBlend=choice.blend.toFixed(3);
  }
}
