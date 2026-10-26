// Local presentation geometry. No workspace data, remote assets or random frames.
export type SharkPoint = { x: number; y: number; z: number; nx: number; ny: number; nz: number; ink?: number; detail?: number };
export type SharkPose = { x: number; y: number; size: number; yaw: number; bank: number; pitch: number; phase: number; opacity: number; tailAmplitude?: number; bend?: number; depth?: number };
export type FishKind = 'shark' | 'compact' | 'slender';
export type SharkGeometry = { points: Float32Array; count: number; kind: FishKind; spine: Float32Array; anchor: number; step: number };
export const OCEAN_ACTORS = [{kind:'shark',size:84},{kind:'shark',size:65},{kind:'compact',size:48},{kind:'slender',size:43}] as const;
// Independent loop regions keep the school spread across the cover. The small
// fish explore its upper/lower areas while the sharks have room on either side.
const SWIM_ROUTES = [
  {cx:.23,cy:.57,rx:.16,ry:.21,phase:.62},
  {cx:.77,cy:.43,rx:.16,ry:.21,phase:2.78},
  {cx:.24,cy:.22,rx:.17,ry:.13,phase:1.55},
  {cx:.76,cy:.78,rx:.17,ry:.13,phase:4.9},
] as const;
export function oceanFit(width:number,height:number){return Math.min(1,width/900,height/288);}
const STRIDE = 10, BONES = 80;
const ELEVATION = Math.PI * .43, CE = Math.cos(ELEVATION), SE = Math.sin(ELEVATION);
const profile = [[-2,.045],[-1.5,.085],[-.85,.19],[-.1,.39],[.48,.445],[1.08,.335],[1.55,.19],[1.88,.065],[1.98,.008]];
function bodyRadius(x: number) {
  for (let i=1;i<profile.length;i++) if (x<=profile[i][0]) {
    const [a,r0]=profile[i-1], [b,r1]=profile[i], t=Math.max(0,(x-a)/(b-a));
    return r0+(r1-r0)*t*t*(3-2*t);
  }
  return profile.at(-1)![1];
}
export function createSharkMesh(): SharkPoint[] {
  const points: SharkPoint[]=[];
  for(let ring=0;ring<=128;ring++) {
    const x=-2+ring/128*3.98, radius=bodyRadius(x);
    const slope=(bodyRadius(x+.008)-bodyRadius(x-.008))/.016;
    for(let j=0;j<64;j++) {
      const a=j/64*Math.PI*2, ny=Math.sin(a)/.77, nz=Math.cos(a), length=Math.hypot(slope,ny,nz);
      points.push({x,y:Math.sin(a)*radius*.77,z:Math.cos(a)*radius,nx:-slope/length,ny:ny/length,nz:nz/length});
    }
  }
  function fin(a:number[],b:number[],c:number[],normal:number[],bend=0) {
    const n=24, length=Math.hypot(...normal);
    for(let i=0;i<=n;i++) for(let j=0;j<=n-i;j++) {
      const u=i/n,v=j/n,w=1-u-v;
      points.push({x:a[0]*u+b[0]*v+c[0]*w,y:a[1]*u+b[1]*v+c[1]*w+bend*u*v*w,z:a[2]*u+b[2]*v+c[2]*w,nx:normal[0]/length,ny:normal[1]/length,nz:normal[2]/length});
    }
  }
  fin([.48,.28,0],[-.65,.19,0],[-.24,1.19,0],[.12,.2,.95]);
  fin([-.96,.09,0],[-1.34,.06,0],[-1.24,.38,0],[.1,.25,.9]);
  for(const side of [-1,1]) {
    fin([.56,-.1,side*.32],[-.37,-.17,side*.29],[-.65,-.25,side*1.05],[.1,.7,side*.55],-.6);
    fin([-.72,-.1,side*.18],[-1.23,-.09,side*.13],[-1.3,-.23,side*.5],[.1,.7,side*.5]);
  }
  // Slight caudal roll keeps both lobes legible from the overhead camera.
  fin([-1.99,.01,0],[-2.88,1.05,.48],[-2.28,.08,0],[0,.65,.76]);
  fin([-2,0,0],[-2.7,-.66,-.38],[-2.27,-.05,0],[0,.65,.76]);
  for(const side of [-1,1]) {
    for(let g=0;g<5;g++) for(let j=0;j<15;j++) {
      const x=.42+g*.115, y=-.13+j*.023, r=bodyRadius(x);
      const z=side*(r*Math.sqrt(Math.max(0,1-(y/(r*.77))**2))+.008);
      points.push({x,y,z,nx:0,ny:0,nz:side,ink:.72,detail:1});
    }
    const x=1.37,y=.07,r=bodyRadius(x);
    points.push({x,y,z:side*(r*Math.sqrt(1-(y/(r*.77))**2)+.012),nx:0,ny:0,nz:side,ink:1,detail:2});
    for(let j=0;j<8;j++) {
      const x=1.47+j*.038,r=bodyRadius(x),y=-.055;
      points.push({x,y,z:side*(r*Math.sqrt(Math.max(0,1-(y/(r*.77))**2))+.01),nx:0,ny:0,nz:side,ink:.6,detail:3});
    }
  }
  return points;
}
function swimmingPose(width:number,height:number,seconds:number,index:number):SharkPose {
  const fit=oceanFit(width,height),shark=index<2,route=SWIM_ROUTES[index];
  const direction=index===1||index===3?-1:1;
  const cadence=seconds*(shark?.55:.7)+index*1.9;
  const omega=[.145,.12,.21,.18][index],tempo=shark?.55:.7;
  const travel=route.phase+direction*omega*(seconds+.15/tempo*(Math.sin(cadence)-Math.sin(index*1.9)));
  const rx=width*route.rx,ry=height*route.ry;
  // Slow into a turn; a wide cover must not make the fish flip at each end.
  const ease=Math.sqrt(Math.max(.08,ry/Math.max(1,rx)));
  const angle=Math.atan2(ease*Math.sin(travel),Math.cos(travel));
  const dx=-rx*Math.sin(angle),dy=ry/SE*Math.cos(angle),length=Math.hypot(dx,dy);
  const curvature=rx*ry/SE/Math.max(1,length**3);
  const depth=[.12,.62,.26,.76][index]+.09*Math.sin(angle+index);
  const size=OCEAN_ACTORS[index].size*fit*(1.04-depth*.13);
  const effort=.5+.5*Math.cos(cadence);
  return {
    x:width*route.cx+rx*Math.cos(angle),
    y:height*route.cy+ry*Math.sin(angle),
    size,yaw:Math.atan2(dy*direction,dx*direction),
    bend:Math.min(shark?2.25:1.4,curvature*size*(shark?3.5:2.3))*direction,
    bank:-direction*Math.min(.16,curvature*size*.13),pitch:Math.sin(cadence)*.015,
    phase:seconds*[4.4,4.9,6.9,8.4][index]+.32*Math.sin(cadence)+index*2.1,
    tailAmplitude:shark?.36+.2*effort:index===2?.23+.08*effort:.36+.1*effort,
    depth,opacity:1-depth*.30,
  };
}
export function sharkPoses(width:number,height:number,seconds:number):SharkPose[] {
  return [0,1,2].map(index=>swimmingPose(width,height,seconds,index));
}
export function createFishMesh(kind:'compact'|'slender'):SharkPoint[] {
  const compact=kind==='compact',points:SharkPoint[]=[];
  const start=compact?-1.15:-1.65,end=compact?1.12:1.68,yscale=compact?1.03:.92;
  const radius=(x:number)=>{
    const t=Math.max(0,Math.min(1,(x-start)/(end-start)));
    return .025+(compact?.54:.225)*Math.sin(Math.PI*t)**(compact?.72:.9)*(.8+.24*t);
  };
  const rings=compact?72:88;
  for(let ring=0;ring<=rings;ring++) {
    const x=start+ring/rings*(end-start),r=radius(x),slope=(radius(x+.008)-radius(x-.008))/.016;
    for(let j=0;j<48;j++) {
      const a=j/48*Math.PI*2,ny=Math.sin(a)/yscale,nz=Math.cos(a),length=Math.hypot(slope,ny,nz);
      points.push({x,y:Math.sin(a)*r*yscale,z:Math.cos(a)*r,nx:-slope/length,ny:ny/length,nz:nz/length});
    }
  }
  function fin(a:number[],b:number[],c:number[],normal:number[]) {
    const n=16,length=Math.hypot(...normal);
    for(let i=0;i<=n;i++)for(let j=0;j<=n-i;j++){
      const u=i/n,v=j/n,w=1-u-v;
      points.push({x:a[0]*u+b[0]*v+c[0]*w,y:a[1]*u+b[1]*v+c[1]*w,z:a[2]*u+b[2]*v+c[2]*w,nx:normal[0]/length,ny:normal[1]/length,nz:normal[2]/length});
    }
  }
  const tail=start-.64;
  fin([start+.06,0,0],[tail,.48,.30],[start-.39,.02,0],[0,.55,.84]);
  fin([start+.06,0,0],[tail,-.48,-.30],[start-.39,-.02,0],[0,.55,.84]);
  fin([.48,compact?.37:.17,0],[-.47,compact?.37:.15,0],[-.2,compact?.82:.46,0],[.1,.3,.9]);
  for(const side of [-1,1]) {
    fin([.54,-.02,side*(compact?.37:.15)],[-.08,-.12,side*(compact?.43:.13)],[compact?-.32:-.4,-.23,side*(compact?.73:.37)],[.15,.8,side*.35]);
    const x=end*.68,y=.06,r=radius(x);
    points.push({x,y,z:side*(r*Math.sqrt(Math.max(0,1-(y/(r*yscale))**2))+.01),nx:0,ny:0,nz:side,ink:.95,detail:2});
    for(let g=0;g<3;g++)for(let j=0;j<8;j++){
      const x=end*.45-g*.08,y=-.06+j*.02,r=radius(x);
      points.push({x,y,z:side*(r*Math.sqrt(Math.max(0,1-(y/(r*yscale))**2))+.005),nx:0,ny:0,nz:side,ink:.65,detail:1});
    }
  }
  return points;
}
export function oceanPoses(width:number,height:number,seconds:number):SharkPose[] {
  return OCEAN_ACTORS.map((_,index)=>swimmingPose(width,height,seconds,index));
}
export function compileSharkMesh(mesh:SharkPoint[],kind:FishKind='shark'):SharkGeometry {
  const points=new Float32Array(mesh.length*STRIDE);
  const anchor=kind==='compact'?.55:.9,end=kind==='compact'?-1.85:kind==='slender'?-2.3:-2.9;
  const step=(anchor-end)/BONES;
  mesh.forEach((p,index)=>{
    const i=index*STRIDE;
    const along=Math.max(0,Math.min(BONES,(anchor-p.x)/step)),bone=Math.min(BONES-1,Math.floor(along));
    points.set([p.x,p.y,p.z,p.nx,p.ny,p.nz,p.x>anchor?-1:bone,along-bone,p.ink??-1,p.detail??0],i);
  });
  return {points,count:mesh.length,kind,spine:new Float32Array((BONES+1)*4),anchor,step};
}
// Integrate unit tangents, rather than sliding the tail sideways and stretching it.
function articulate(geometry:SharkGeometry,pose:SharkPose) {
  const {spine,anchor,step,kind}=geometry;
  const bend=pose.bend??0,amplitude=pose.tailAmplitude??.32,wavelength=kind==='shark'?4.8:5.8;
  spine.set([anchor,0,1,0]);
  let x=anchor,z=0;
  function angle(u:number){return bend*u*(.7+.3*u)+Math.sin(pose.phase-u*wavelength)*amplitude*u**1.65;}
  for(let bone=1;bone<=BONES;bone++){
    const mid=angle((bone-.5)/BONES),a=angle(bone/BONES);
    x-=Math.cos(mid)*step;z-=Math.sin(mid)*step;
    spine.set([x,z,Math.cos(a),Math.sin(a)],bone*4);
  }
}
function projection(pose:SharkPose) {
  return {pose,cy:Math.cos(pose.yaw),sy:Math.sin(pose.yaw),cb:Math.cos(pose.bank),sb:Math.sin(pose.bank),cp:Math.cos(pose.pitch),sp:Math.sin(pose.pitch)};
}
type Projection=ReturnType<typeof projection>;
function writePoint(x:number,y:number,z:number,nx:number,ny:number,nz:number,ink:number,p:Projection,out:Float32Array,offset:number) {
  const by=y*p.cb-z*p.sb,bz=y*p.sb+z*p.cb;
  const px=x*p.cy+bz*p.sy,pz=-x*p.sy+bz*p.cy;
  const py=(by*CE+pz*SE)*p.cp+px*p.sp,depth=pz*CE-by*SE,perspective=7/(7+depth);
  out[offset]=p.pose.x+px*p.pose.size*perspective;
  out[offset+1]=p.pose.y-py*p.pose.size*perspective;
  out[offset+2]=depth;
  out[offset+3]=ink>=0?ink:Math.max(.08,Math.min(1,.30+(ny*p.cb-nz*p.sb)*.45+(-nx*p.sy+(ny*p.sb+nz*p.cb)*p.cy)*.22-(nx*p.cy+(ny*p.sb+nz*p.cb)*p.sy)*.16));
}
export function projectSharkMesh(geometry:SharkGeometry,pose:SharkPose,out:Float32Array) {
  articulate(geometry,pose);
  const p=projection(pose),points=geometry.points,spine=geometry.spine;
  for(let n=0;n<geometry.count;n++) {
    const i=n*STRIDE,bone=points[i+6];
    let x=points[i],z=points[i+2],nx=points[i+3],nz=points[i+5];
    if(bone>=0){
      const b=bone*4,t=points[i+7];
      const cx=spine[b]+(spine[b+4]-spine[b])*t,cz=spine[b+1]+(spine[b+5]-spine[b+1])*t;
      const c=spine[b+2]+(spine[b+6]-spine[b+2])*t,s=spine[b+3]+(spine[b+7]-spine[b+3])*t;
      x=cx-z*s;z=cz+z*c;
      const normalX=nx;nx=normalX*c-nz*s;nz=normalX*s+nz*c;
    }
    writePoint(x,points[i+1],z,nx,points[i+4],nz,points[i+8],p,out,n*4);
  }
}
export function sharkDetail(geometry:SharkGeometry,index:number) { return geometry.points[index*STRIDE+9]; }
export function projectShark(point:SharkPoint,pose:SharkPose) {
  const out=new Float32Array(4);
  projectSharkMesh(compileSharkMesh([point]),pose,out);
  return {x:out[0],y:out[1],depth:out[2],light:out[3]};
}
