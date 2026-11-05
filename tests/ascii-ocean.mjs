import test from 'node:test';
import assert from 'node:assert/strict';
import { OceanClock } from '../src/lib/ocean-clock.ts';
import { createSharkMesh, createFishMesh, compileSharkMesh, projectSharkMesh, projectShark, sharkPoses, oceanPoses, OCEAN_ACTORS } from '../src/lib/shark-scene.ts';

test('active time stays consistent from 30 to 240Hz while paint is capped at 120',()=>{
  for(const hz of [30,60,90,120,144,240]){
    const clock=new OceanClock();let painted=0,last;
    for(let i=0;i<=hz*4;i++){const frame=clock.tick(i*1000/hz);if(frame){painted++;last=frame;}}
    assert(Math.abs(painted-(Math.min(hz,120)*4+1))<=1,`${hz}Hz: ${painted} frames`);
    assert(Math.abs(last.seconds-4)<.00001,`${hz}Hz: ${last.seconds}s`);
  }
});
test('suspension and static reduced motion exclude hidden time',()=>{
  const clock=new OceanClock();clock.tick(0);const before=clock.tick(1000/60);
  clock.suspend();assert.deepEqual(clock.tick(1_000_000,true),{seconds:0,delta:0,frame:0});
  clock.suspend();const resumed=clock.tick(2_000_000);
  assert.equal(resumed.seconds,before.seconds);assert.equal(resumed.delta,0);
  assert(Math.abs(clock.tick(2_000_000+1000/120).delta-1/120)<1e-10);
});
test('an interrupted active callback cannot jump across minutes',()=>{
  const clock=new OceanClock();clock.tick(0);assert.equal(clock.tick(600_000).seconds,.1);
});
test('compiled silhouette remains finite through all headings and matches sampled surface points',()=>{
  const mesh=createSharkMesh(),geometry=compileSharkMesh(mesh),out=new Float32Array(mesh.length*4);
  for(const [width,height,time] of [[1600,288,0],[420,230,12],[900,288,40],[2400,288,87]]){
    for(const pose of sharkPoses(width,height,time)){
      projectSharkMesh(geometry,pose,out);
      assert(out.every(Number.isFinite));
      const xs=[],ys=[];
      for(let n=0;n<mesh.length;n+=97){
        const point=projectShark(mesh[n],pose);xs.push(out[n*4]);ys.push(out[n*4+1]);
        assert(Math.abs(out[n*4]-point.x)<.001);assert(Math.abs(out[n*4+1]-point.y)<.001);
      }
      assert(Math.max(...xs)>Math.min(...xs));assert(Math.max(...ys)>Math.min(...ys));
    }
  }
});
test('tail deforms while the nose stays steady within a pose',()=>{
  const pose={x:0,y:0,size:1,yaw:0,bank:0,pitch:0,phase:0,opacity:1};
  const nose={x:1.7,y:0,z:0,nx:1,ny:0,nz:0},tail=createSharkMesh().reduce((tip,p)=>p.x<tip.x?p:tip);
  const shifted={...pose,phase:Math.PI/2};
  const distance=p=>Math.hypot(projectShark(p,pose).x-projectShark(p,shifted).x,projectShark(p,pose).y-projectShark(p,shifted).y);
  assert.equal(distance(nose),0);assert(distance(tail)>.1);
});
test('compact and slender fish have different proportions rather than scaled copies',()=>{
  const compact=createFishMesh('compact'),slender=createFishMesh('slender');
  const span=(mesh,key)=>Math.max(...mesh.map(p=>p[key]))-Math.min(...mesh.map(p=>p[key]));
  assert(span(compact,'x')<span(slender,'x')*.85);
  assert(span(compact,'z')>span(slender,'z')*1.5);
  for(const [kind,mesh] of [['compact',compact],['slender',slender]]){
    const geometry=compileSharkMesh(mesh,kind),out=new Float32Array(mesh.length*4);
    for(const time of [0,2,8,16,24]){
      projectSharkMesh(geometry,oceanPoses(1600,288,time)[kind==='compact'?2:3],out);
      assert(out.every(Number.isFinite));
    }
  }
});
test('different fish gaits preserve steady heads with independent tail cadence',()=>{
  const before=oceanPoses(1600,288,3),after=oceanPoses(1600,288,3.1);
  const advances=after.map((pose,i)=>pose.phase-before[i].phase);
  assert(Math.abs(advances[2]-advances[3])>.1);
  assert(Math.abs(advances[0]-advances[2])>.1);
  assert(before[2].tailAmplitude<before[3].tailAmplitude);
  for(const pose of after)assert(Object.values(pose).every(Number.isFinite));
});
test('third shark stays smaller, deeper and quieter than both foreground sharks',()=>{
  const sharks=OCEAN_ACTORS.flatMap((actor,i)=>actor.kind==='shark'?[i]:[]);
  assert.equal(sharks.length,3);
  for(const [width,height] of [[1000,288],[420,230]])for(let seconds=0;seconds<=180;seconds+=.5){
    const poses=oceanPoses(width,height,seconds),background=poses[sharks[2]];
    assert(Object.values(background).every(Number.isFinite));
    assert(sharks.slice(0,2).every(i=>background.size<poses[i].size&&background.opacity<poses[i].opacity));
    assert(poses.every((pose,i)=>i===sharks[2]||background.depth>pose.depth));
    const next=oceanPoses(width,height,seconds+1/120)[sharks[2]];
    assert(Math.hypot(next.x-background.x,next.y-background.y)<width*.002);
  }
});
