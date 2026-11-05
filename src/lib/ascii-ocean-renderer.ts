import { compileSharkMesh, createSharkMesh, createFishMesh, projectSharkMesh, sharkDetail, oceanPoses, oceanFit, OCEAN_ACTORS, type SharkGeometry, type SharkPose } from './shark-scene';
import { SharkHabitatRenderer } from './shark-habitat';
const CW=6,CH=8,GLYPHS=['.',':','o',';','!','t','+','S','X','%','8','#','@','_','-'];
const SHADES=11,STYLES=17;
const DAY=[
  ['#363c48','#48505e','#5d6674','#737e8b','#89939f','#9fa8b3','#b2bac3','#c4cad1','#d5d9df','#e4e7ec','#f0f1f4'],
  ['#433e30','#57503d','#6c654c','#817a5e','#969076','#aaa58d','#bcb8a3','#cdc9b8','#dcd9cc','#e8e5dc','#f1efe8'],
  ['#474133','#5b5440','#71684e','#877e60','#9d9376','#b1a78c','#c2baa1','#d2cbb6','#e0dacc','#eae6dd','#f3f0e9'],
];
const NIGHT=[
  ['#141720','#202530','#303846','#424c5c','#566174','#6b768a','#8490a2','#9ea8b7','#b7bfcc','#cfd4de','#e5e8ee'],
  ['#191811','#29271c','#3b3929','#504c36','#676149','#80785b','#999172','#b2a98c','#c9c2a7','#dfd8c2','#eee9da'],
  ['#1c1911','#2e291c','#423b28','#584e35','#706347','#8a7a58','#a49370','#bdad8b','#d2c4a5','#e3d8bf','#f0e8d6'],
];
const ACCENTS=['#959fb7','#aaa17c','#83a28c','#b19bb7','#73a293','#a1a9bf'];
const TONES=Array.from({length:SHADES},(_,level)=>Math.round((level/(SHADES-1))**1.35*(SHADES-1)));
function context(canvas:HTMLCanvasElement){const ctx=canvas.getContext('2d');if(!ctx)throw Error('Canvas2D unavailable');return ctx;}
function rgb(color:string){const n=parseInt(color.slice(1),16);return [(n>>16)&255,(n>>8)&255,n&255];}
function hash(seed:number){const n=Math.sin(seed*127.1+17.7)*43758.5453;return n-Math.floor(n);}
class GlyphAtlas {
  readonly canvas=document.createElement('canvas');
  private tiles=document.createElement('canvas');
  constructor(readonly dpr:number,dark:boolean,ink:string){
    this.canvas.width=CW*dpr*GLYPHS.length;this.canvas.height=CH*dpr;
    const plain=context(this.canvas);plain.scale(dpr,dpr);plain.font='400 8px ui-monospace, SFMono-Regular, Menlo, monospace';plain.textBaseline='alphabetic';plain.fillStyle=ink;
    GLYPHS.forEach((glyph,index)=>plain.fillText(glyph,index*CW+.5,6.5));
    this.tiles.width=this.canvas.width;this.tiles.height=CH*dpr*STYLES*3;
    const ctx=context(this.tiles);ctx.scale(dpr,dpr);ctx.font=plain.font;ctx.textBaseline='alphabetic';
    for(let family=0;family<3;family++)for(let style=0;style<STYLES;style++){
      const fill=style<SHADES?(dark?NIGHT:DAY)[family][style]:ACCENTS[style-SHADES];
      const [r,g,b]=rgb(fill),light=r*.299+g*.587+b*.114;
      const foreground=light<(dark?111:152)?(dark?'#c1c5c9':'#ecece6'):'#333945';
      const y=(family*STYLES+style)*CH;
      for(let index=0;index<GLYPHS.length;index++){
        const x=index*CW;ctx.fillStyle=fill;ctx.fillRect(x+.3,y+.3,CW-.6,CH-.6);ctx.fillStyle=foreground;ctx.fillText(GLYPHS[index],x+.5,y+6.5);
      }
    }
  }
  draw(ctx:CanvasRenderingContext2D,index:number,x:number,y:number,alpha:number){
    ctx.globalAlpha=alpha;ctx.drawImage(this.canvas,index*CW*this.dpr,0,CW*this.dpr,CH*this.dpr,x,y,CW,CH);
  }
  tile(ctx:CanvasRenderingContext2D,index:number,level:number,accent:number,family:number,x:number,y:number,alpha:number){
    const row=family*STYLES+(accent<0?level:SHADES+accent);ctx.globalAlpha=alpha;
    ctx.drawImage(this.tiles,index*CW*this.dpr,row*CH*this.dpr,CW*this.dpr,CH*this.dpr,x,y,CW,CH);
  }
}
// Local grids travel with the animals, so letters do not jump on translation.
class FishLayer {
  readonly canvas=document.createElement('canvas');
  private ctx:CanvasRenderingContext2D;private cols:number;private rows:number;
  private depth:Float32Array;private light:Float32Array;private smooth:Float32Array;
  private levels:Int8Array;private occupied:Uint8Array;private detail:Uint8Array;
  private projected:Float32Array;private family:number;
  readonly width:number;readonly height:number;
  constructor(private geometry:SharkGeometry,size:number,private dpr:number){
    this.width=Math.ceil(size*8/CW)*CW;this.height=Math.ceil(size*8/CH)*CH;
    this.cols=Math.ceil(this.width/CW);this.rows=Math.ceil(this.height/CH);const count=this.cols*this.rows;
    this.depth=new Float32Array(count);this.light=new Float32Array(count);this.smooth=new Float32Array(count);
    this.levels=new Int8Array(count).fill(-1);this.occupied=new Uint8Array(count);this.detail=new Uint8Array(count);
    this.projected=new Float32Array(geometry.count*4);this.family=geometry.kind==='compact'?1:geometry.kind==='slender'?2:0;
    this.canvas.width=Math.ceil(this.width*dpr);this.canvas.height=Math.ceil(this.height*dpr);this.ctx=context(this.canvas);
  }
  draw(pose:SharkPose,delta:number,atlas:GlyphAtlas){
    const p=this.projected;projectSharkMesh(this.geometry,{...pose,x:this.width/2,y:this.height/2},p);
    this.depth.fill(Infinity);this.detail.fill(0);
    for(let n=0;n<this.geometry.count;n++){
      const offset=n*4,col=Math.floor(p[offset]/CW),row=Math.floor(p[offset+1]/CH);
      if(col<0||col>=this.cols||row<0||row>=this.rows)continue;
      const i=row*this.cols+col,feature=sharkDetail(this.geometry,n),depth=p[offset+2]-(feature ? .07 : 0);
      if(depth<this.depth[i]){this.depth[i]=depth;this.light[i]=p[offset+3];this.detail[i]=feature;}
    }
    const ctx=this.ctx;ctx.setTransform(this.dpr,0,0,this.dpr,0,0);ctx.clearRect(0,0,this.width,this.height);
    const blend=delta>0?1-Math.exp(-delta*18):1;
    for(let i=0;i<this.depth.length;i++){
      if(this.depth[i]===Infinity){this.occupied[i]=0;this.levels[i]=-1;continue;}
      const difference=this.light[i]-this.smooth[i],shade=this.occupied[i]&&Math.abs(difference)>.0025?this.smooth[i]+difference*blend:this.light[i];
      this.occupied[i]=1;this.smooth[i]=shade;
      const scaled=shade*(SHADES-1),old=this.levels[i];let level=old;
      if(old<0)level=Math.round(scaled);else if(scaled>old+.68)level=Math.floor(scaled+.32);else if(scaled<old-.68)level=Math.ceil(scaled-.32);
      level=Math.max(0,Math.min(SHADES-1,level));this.levels[i]=level;
      const seed=hash(i*7+this.family*97),feature=this.detail[i],tone=TONES[level];
      const rank=Math.max(0,Math.min(12,Math.round(2+(1-tone/(SHADES-1))*9)+Math.floor(seed*5)-2));
      const glyph=feature===2?2:feature===1?3:feature===3?13:rank;
      const accent=seed>.982&&!feature?Math.floor(hash(i+997)*ACCENTS.length):-1;
      const col=i%this.cols,row=Math.floor(i/this.cols),edge=col===0||col===this.cols-1||row===0||row===this.rows-1||this.depth[i-1]===Infinity||this.depth[i+1]===Infinity||this.depth[i-this.cols]===Infinity||this.depth[i+this.cols]===Infinity;
      atlas.tile(ctx,glyph,tone,accent,this.family,col*CW,row*CH,feature===2?.98:edge?.61:.94);
    }
    ctx.globalAlpha=1;
  }
}
export class AsciiOceanRenderer {
  private shark=compileSharkMesh(createSharkMesh());
  private geometries=OCEAN_ACTORS.map(actor=>actor.kind==='shark'?this.shark:compileSharkMesh(createFishMesh(actor.kind),actor.kind));
  private ctx:CanvasRenderingContext2D;private layers:FishLayer[]=[];private atlas:GlyphAtlas|null=null;
  private grid=document.createElement('canvas');private particles=Float32Array.from({length:36*4},(_,i)=>hash(i+13));
  private school=Float32Array.from({length:120*4},(_,i)=>hash(i+613));
  private order=OCEAN_ACTORS.map((_,index)=>index);
  private habitat=new SharkHabitatRenderer();
  private width=0;private height=0;private dpr=1;private dark=true;private ink='';private seconds=0;
  constructor(private canvas:HTMLCanvasElement){this.ctx=context(canvas);}
  resize(width:number,height:number,dpr:number,dark:boolean,ink:string){
    width=Math.max(0,width);height=Math.max(0,height);dpr=Math.min(2,Math.max(1,dpr));
    if(width===this.width&&height===this.height&&dpr===this.dpr&&dark===this.dark&&ink===this.ink&&this.atlas)return;
    const atlasChanged=!this.atlas||dpr!==this.dpr||dark!==this.dark||ink!==this.ink;
    this.width=width;this.height=height;this.dpr=dpr;this.dark=dark;this.ink=ink;
    this.canvas.width=Math.round(width*dpr);this.canvas.height=Math.round(height*dpr);
    if(atlasChanged)this.atlas=new GlyphAtlas(dpr,dark,ink);
    const fit=oceanFit(width,height);this.layers=fit>0?OCEAN_ACTORS.map((actor,i)=>new FishLayer(this.geometries[i],actor.size*fit*1.06,dpr)):[];
    this.grid.width=Math.ceil(width*dpr);this.grid.height=Math.ceil(height*dpr);const ctx=context(this.grid);ctx.scale(dpr,dpr);ctx.strokeStyle=ink;ctx.lineWidth=.5;
    ctx.globalAlpha=dark?.028:.035;ctx.beginPath();for(let x=0;x<width;x+=12){ctx.moveTo(x+.5,0);ctx.lineTo(x+.5,height);}for(let y=0;y<height;y+=12){ctx.moveTo(0,y+.5);ctx.lineTo(width,y+.5);}ctx.stroke();
    ctx.globalAlpha=dark?.02:.025;ctx.beginPath();for(let x=0;x<width;x+=60){ctx.moveTo(x+.5,0);ctx.lineTo(x+.5,height);}for(let y=0;y<height;y+=60){ctx.moveTo(0,y+.5);ctx.lineTo(width,y+.5);}ctx.stroke();
    this.habitat.resize(width,height);
    // ResizeObserver runs before paint: restore the cleared bitmap in this turn,
    // using the last scene time rather than exposing a blank frame until RAF.
    this.draw(this.seconds,0);
  }
  draw(seconds:number,delta:number){
    this.seconds=seconds;
    if(!this.width||!this.height||!this.atlas)return;
    const ctx=this.ctx,w=this.width,h=this.height,atlas=this.atlas;ctx.setTransform(this.dpr,0,0,this.dpr,0,0);ctx.clearRect(0,0,w,h);
    ctx.globalAlpha=1;ctx.drawImage(this.grid,0,0,w,h);
    this.habitat.draw(ctx,seconds,atlas);
    for(let i=0;i<36;i++){
      const j=i*4,depth=this.particles[j+2],x=(this.particles[j]*w+seconds*(1.5+depth*3))%(w+24)-12;
      const y=(this.particles[j+1]*h-seconds*(.7+depth*1.3)+h*100)%h;
      atlas.draw(ctx,depth>.7?1:0,x+Math.sin(seconds*.22+i)*3,y,.045+depth*.12);
    }
    for(let i=0;i<120;i++){
      const j=i*4,group=Math.floor(i/40),direction=group===1?-1:1;
      const progress=(seconds*(5+group*2)+w*[.1,.52,.83][group])%(w+220)-110;
      const x=(direction>0?progress:w-progress)+(this.school[j]-.5)*150;
      const y=h*[.25,.75,.38][group]+(this.school[j+1]-.5)*56+Math.sin(seconds*.45+this.school[j+2]*6)*3;
      const edge=Math.min(1,Math.max(0,Math.min(x,w-x)/40));
      atlas.draw(ctx,this.school[j+3]>.7?14:0,x,y,(.09+this.school[j+2]*.16)*edge);
    }
    const poses=oceanPoses(w,h,seconds);
    this.order.sort((a,b)=>(poses[b].depth??0)-(poses[a].depth??0));
    for(const i of this.order){const pose=poses[i],layer=this.layers[i];layer.draw(pose,delta,atlas);ctx.globalAlpha=pose.opacity;ctx.drawImage(layer.canvas,pose.x-layer.width/2,pose.y-layer.height/2,layer.width,layer.height);}
    ctx.globalAlpha=1;
  }
}
