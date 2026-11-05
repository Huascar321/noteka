// Local procedural silhouettes inspired by the supplied koi/Kurage references.
// No third-party source, fonts, interfaces or network resources are embedded.
import { sceneNoise, koiBody, koiPoses, jellyPoses, type KoiPose, type JellyPose } from './ascii-cover-scene';
import { KoiHabitatRenderer, backgroundKoiPoses } from './koi-habitat';
import { JellyHabitatRenderer, backgroundJellyPoses } from './jelly-habitat';
const CW=6, CH=8, LEVELS=11, GLYPHS=['.',':',';','o','+','t','S','X','%','8','#','@'];
const DARK=[['#373b40','#f4eee3'],['#753018','#ffba77'],['#123644','#83f0ed'],['#342346','#c2a4ff'],['#15161b','#626772']];
const LIGHT=[['#a6a6a0','#f5eee0'],['#a43716','#f7aa65'],['#163a49','#4d989a'],['#39264e','#916bb0'],['#171a20','#61636b']];
function context(canvas:HTMLCanvasElement,read=false){const ctx=canvas.getContext('2d',read?{willReadFrequently:true}:undefined);if(!ctx)throw Error('Canvas2D unavailable');return ctx;}
function interpolate(a:string,b:string,t:number){const n=parseInt(a.slice(1),16),m=parseInt(b.slice(1),16);return `rgb(${[16,8,0].map(s=>Math.round(((n>>s)&255)*(1-t)+((m>>s)&255)*t)).join(',')})`;}
function signal(family:number,light:number,alpha=1){return `rgba(${20+family*40},${Math.round(light)},0,${alpha})`;}
class CreatureAtlas {
  readonly canvas=document.createElement('canvas');
  constructor(readonly dpr:number,dark:boolean){
    this.canvas.width=GLYPHS.length*CW*dpr;this.canvas.height=5*LEVELS*CH*dpr;
    const ctx=context(this.canvas);ctx.scale(dpr,dpr);ctx.font='400 8px ui-monospace, SFMono-Regular, Menlo, monospace';ctx.textBaseline='alphabetic';
    for(let family=0;family<5;family++)for(let level=0;level<LEVELS;level++){
      const y=(family*LEVELS+level)*CH,palette=(dark?DARK:LIGHT)[family],color=interpolate(palette[0],palette[1],level/(LEVELS-1));
      for(let glyph=0;glyph<GLYPHS.length;glyph++){
        const x=glyph*CW;ctx.globalAlpha=family===2||family===3?.20:.94;ctx.fillStyle=color;ctx.fillRect(x+.3,y+.3,CW-.6,CH-.6);
        ctx.globalAlpha=1;ctx.fillStyle=family===2||family===3?color:family===4?'#d4d2cd':level>4?'#303239':dark?'#e4ded4':'#fcf5e9';ctx.fillText(GLYPHS[glyph],x+.5,y+6.5);
      }
    }ctx.globalAlpha=1;
  }
  draw(ctx:CanvasRenderingContext2D,family:number,level:number,glyph:number,x:number,y:number,alpha:number){
    ctx.globalAlpha=alpha;ctx.drawImage(this.canvas,glyph*CW*this.dpr,(family*LEVELS+level)*CH*this.dpr,CW*this.dpr,CH*this.dpr,x,y,CW,CH);
  }
}
export class AsciiCreaturesRenderer {
  private ctx:CanvasRenderingContext2D;private field=document.createElement('canvas');private fieldCtx=context(this.field,true);
  private grid=document.createElement('canvas');
  private atlas:CreatureAtlas|null=null;private width=0;private height=0;private cols=0;private rows=0;private dpr=1;private dark=true;
  private habitat:KoiHabitatRenderer|JellyHabitatRenderer;
  constructor(private canvas:HTMLCanvasElement,private kind:'koi'|'jellyfish',private seed:number){this.ctx=context(canvas);this.habitat=kind==='koi'?new KoiHabitatRenderer(seed):new JellyHabitatRenderer(seed);}
  resize(width:number,height:number,dpr:number,dark:boolean,ink=dark?'#e9e9e7':'#37352f'){
    this.width=Math.max(0,width);this.height=Math.max(0,height);this.cols=Math.ceil(width/CW);this.rows=Math.ceil(height/CH);
    this.dpr=Math.min(2,Math.max(1,dpr));if(!this.atlas||this.atlas.dpr!==this.dpr||this.dark!==dark)this.atlas=new CreatureAtlas(this.dpr,dark);this.dark=dark;
    this.canvas.width=Math.round(width*this.dpr);this.canvas.height=Math.round(height*this.dpr);this.field.width=this.cols*2;this.field.height=this.rows*2;
    this.grid.width=Math.ceil(width*this.dpr);this.grid.height=Math.ceil(height*this.dpr);
    const grid=context(this.grid);grid.scale(this.dpr,this.dpr);grid.strokeStyle=ink;grid.lineWidth=.5;
    grid.globalAlpha=dark?.028:.035;grid.beginPath();for(let x=0;x<width;x+=12){grid.moveTo(x+.5,0);grid.lineTo(x+.5,height);}for(let y=0;y<height;y+=12){grid.moveTo(0,y+.5);grid.lineTo(width,y+.5);}grid.stroke();
    grid.globalAlpha=dark?.02:.025;grid.beginPath();for(let x=0;x<width;x+=60){grid.moveTo(x+.5,0);grid.lineTo(x+.5,height);}for(let y=0;y<height;y+=60){grid.moveTo(0,y+.5);grid.lineTo(width,y+.5);}grid.stroke();
  }
  private koi(p:KoiPose){
    const ctx=this.fieldCtx,L=p.length;ctx.save();ctx.translate(p.x,p.y);ctx.rotate(p.heading);
    const tail=koiBody(1,L,p.phase);ctx.fillStyle=signal(0,210,.58);ctx.strokeStyle=signal(0,230,.7);ctx.lineWidth=1.2;
    ctx.beginPath();ctx.moveTo(tail.x,tail.y);ctx.bezierCurveTo(tail.x-L*.10,tail.y-L*.10,tail.x-L*.23,tail.y-L*.19,tail.x-L*.24,tail.y-L*.17);ctx.quadraticCurveTo(tail.x-L*.17,tail.y,tail.x-L*.24,tail.y+L*.17);ctx.bezierCurveTo(tail.x-L*.20,tail.y+L*.19,tail.x-L*.08,tail.y+L*.08,tail.x,tail.y);ctx.fill();ctx.stroke();
    for(const sign of [-1,1]){ctx.beginPath();ctx.moveTo(L*.22,sign*L*.08);ctx.bezierCurveTo(L*.07,sign*L*.27,-L*.12,sign*L*.26,L*.01,sign*L*.10);ctx.fill();}
    const points=Array.from({length:30},(_,i)=>koiBody(i/29,L,p.phase));
    ctx.beginPath();ctx.moveTo(points[0].x,points[0].y-points[0].radius);for(const q of points)ctx.lineTo(q.x,q.y-q.radius);for(const q of [...points].reverse())ctx.lineTo(q.x,q.y+q.radius);ctx.closePath();
    const shade=ctx.createLinearGradient(0,-L*.15,0,L*.15);shade.addColorStop(0,signal(0,125));shade.addColorStop(.4,signal(0,245));shade.addColorStop(1,signal(0,150));ctx.fillStyle=shade;ctx.fill();ctx.save();ctx.clip();
    for(let i=0;i<4;i++){const u=.08+i*.23,q=koiBody(u,L,p.phase),offset=(sceneNoise(p.seed,i)-.5)*L*.06;ctx.fillStyle=signal(1,165+sceneNoise(p.seed,i+6)*70);ctx.beginPath();ctx.ellipse(q.x+offset,q.y+offset*.6,L*(.07+sceneNoise(p.seed,i+2)*.055),L*.12,(sceneNoise(p.seed,i+4)-.5)*.7,0,Math.PI*2);ctx.fill();}
    ctx.restore();for(const sign of [-1,1]){ctx.fillStyle=signal(4,50);ctx.beginPath();ctx.arc(L*.36,sign*L*.045,L*.017,0,Math.PI*2);ctx.fill();ctx.strokeStyle=signal(0,200,.8);ctx.beginPath();ctx.moveTo(L*.45,sign*L*.018);ctx.lineTo(L*.50,sign*L*.060);ctx.stroke();}ctx.restore();
  }
  private jelly(p:JellyPose){
    const ctx=this.fieldCtx,R=p.radius,rx=R*(1-.13*p.pulse),ry=R*(.74+.14*p.pulse);ctx.save();ctx.translate(p.x,p.y);ctx.rotate(p.angle);ctx.lineCap='round';
    for(let i=0;i<11;i++){
      const anchor=(i/10-.5)*rx*1.7,length=R*(1.65+sceneNoise(p.seed,i)*.85);ctx.beginPath();
      for(let n=0;n<=22;n++){const u=n/22,x=anchor+Math.sin(p.phase-u*5.2+i*.67)*R*.20*u**1.3,y=length*u; if(n)ctx.lineTo(x,y);else ctx.moveTo(x,y);}
      ctx.lineWidth=R*(i%3===0?.045:.022);ctx.strokeStyle=signal(p.family,170+sceneNoise(p.seed,i+12)*60,.7);ctx.stroke();
    }
    for(let i=0;i<4;i++){
      ctx.beginPath();for(let n=0;n<=18;n++){const u=n/18,x=(i-1.5)*R*.14+Math.sin(p.phase*1.1-u*7+i)*R*.18*u,y=R*1.45*u; if(n)ctx.lineTo(x,y);else ctx.moveTo(x,y);}
      ctx.lineWidth=R*.095;ctx.strokeStyle=signal(p.family===2?3:2,185,.30);ctx.stroke();
    }
    ctx.beginPath();ctx.moveTo(-rx,0);ctx.bezierCurveTo(-rx,-ry*.62,-rx*.48,-ry,0,-ry);ctx.bezierCurveTo(rx*.48,-ry,rx,-ry*.62,rx,0);ctx.quadraticCurveTo(0,-ry*.18,-rx,0);ctx.closePath();
    const glow=ctx.createRadialGradient(0,-ry*.45,R*.08,0,-ry*.35,R);glow.addColorStop(0,signal(p.family,165,.35));glow.addColorStop(.65,signal(p.family,205,.50));glow.addColorStop(1,signal(p.family,250,.88));ctx.fillStyle=glow;ctx.fill();ctx.lineWidth=R*.047;ctx.strokeStyle=signal(p.family,248,.90);ctx.stroke();
    ctx.strokeStyle=signal(p.family===2?3:2,245,.9);ctx.lineWidth=R*.035;
    for(let i=0;i<4;i++){const angle=Math.PI*.25+i*Math.PI*.5;ctx.beginPath();ctx.ellipse(Math.cos(angle)*rx*.23,-ry*.45+Math.sin(angle)*ry*.19,rx*.14,ry*.12,angle,0,Math.PI*2);ctx.stroke();}
    ctx.fillStyle=signal(p.family,245,.9);for(let i=0;i<9;i++){ctx.beginPath();ctx.arc((i/8-.5)*rx*1.8,-ry*.025,R*.026,0,Math.PI*2);ctx.fill();}ctx.restore();
  }
  private prepareField(){
    const field=this.fieldCtx;field.setTransform(1,0,0,1,0,0);field.clearRect(0,0,this.field.width,this.field.height);field.setTransform(2/CW,0,0,2/CH,0,0);
    field.globalAlpha=1;
  }
  private paintField(ctx:CanvasRenderingContext2D){
    if(!this.atlas)return;
    const pixels=this.fieldCtx.getImageData(0,0,this.field.width,this.field.height).data;
    for(let row=0;row<this.rows;row++)for(let col=0;col<this.cols;col++){
      const offset=((row*2+1)*this.field.width+col*2+1)*4,alpha=pixels[offset+3]/255;if(alpha<.06)continue;
      const family=Math.max(0,Math.min(4,Math.round((pixels[offset]-20)/40))),level=Math.max(0,Math.min(10,Math.round(pixels[offset+1]/255*10)));
      const glyph=Math.min(GLYPHS.length-1,Math.floor(sceneNoise(this.seed,col+row*997)*5)+(this.kind==='jellyfish'?Math.round(level*.5):5));
      this.atlas.draw(ctx,family,level,glyph,col*CW,row*CH,!this.dark&&this.kind==='jellyfish'?Math.min(1,alpha*1.45):alpha);
    }ctx.globalAlpha=1;
  }
  draw(seconds:number){
    if(!this.width||!this.height||!this.atlas)return;
    const ctx=this.ctx;ctx.setTransform(this.dpr,0,0,this.dpr,0,0);ctx.clearRect(0,0,this.width,this.height);
    ctx.globalAlpha=1;ctx.drawImage(this.grid,0,0,this.width,this.height);
    this.habitat.draw(ctx,this.width,this.height,seconds,this.dpr,this.dark);
    this.prepareField();
    if(this.kind==='koi'){
      for(const fish of backgroundKoiPoses(this.width,this.height,seconds,this.seed)){
        this.fieldCtx.globalAlpha=fish.opacity;this.koi(fish.pose);
      }
    }else{
      for(const jelly of backgroundJellyPoses(this.width,this.height,seconds,this.seed)){
        this.fieldCtx.globalAlpha=jelly.opacity;this.jelly(jelly.pose);
      }
    }
    this.paintField(ctx);
    this.prepareField();
    if(this.kind==='koi')for(const pose of koiPoses(this.width,this.height,seconds,this.seed))this.koi(pose);else for(const pose of jellyPoses(this.width,this.height,seconds,this.seed))this.jelly(pose);
    this.paintField(ctx);
  }
}
