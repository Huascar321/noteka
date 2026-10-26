import test from 'node:test';
import assert from 'node:assert/strict';
import { COVER_SCENES, SCENE_SECONDS, SCENE_FADE_SECONDS, coverSceneAt, koiPoses, koiBody, jellyPoses } from '../src/lib/ascii-cover-scene.ts';
test('seeded scene order covers all three without consecutive repeats',()=>{
  const firsts=new Set();
  for(let seed=0;seed<48;seed++){
    const order=Array.from({length:9},(_,i)=>coverSceneAt(seed,i*SCENE_SECONDS).current);firsts.add(order[0]);
    assert.deepEqual(new Set(order.slice(0,3)),new Set(COVER_SCENES));
    for(let i=1;i<order.length;i++)assert.notEqual(order[i],order[i-1]);
    assert.deepEqual(coverSceneAt(seed,17),coverSceneAt(seed,17));
  }
  assert.equal(firsts.size,3);
});
test('crossfade is gradual and continuous at the scene boundary',()=>{
  const start=coverSceneAt(17,SCENE_SECONDS-SCENE_FADE_SECONDS),middle=coverSceneAt(17,SCENE_SECONDS-SCENE_FADE_SECONDS/2),end=coverSceneAt(17,SCENE_SECONDS-.00001),next=coverSceneAt(17,SCENE_SECONDS);
  assert.equal(start.blend,0);assert(Math.abs(middle.blend-.5)<1e-10);assert(end.blend>.9999);assert.equal(next.current,end.next);assert.equal(next.blend,0);
  assert.deepEqual(coverSceneAt(17,-1),coverSceneAt(17,0));assert.deepEqual(coverSceneAt(17,NaN),coverSceneAt(17,0));
});
test('koi profile has a steady snout and moving posterior, with finite viewport poses',()=>{
  assert.deepEqual(koiBody(0,150,0),koiBody(0,150,2));assert.notEqual(koiBody(1,150,0).y,koiBody(1,150,2).y);
  assert(koiBody(.4,150,0).radius>koiBody(1,150,0).radius*3);
  for(const [w,h]of[[420,230],[900,288],[2400,288]])for(const t of[0,2,12,42,120])for(const p of koiPoses(w,h,t,3)){
    assert(Object.values(p).every(Number.isFinite));assert(p.length>0);assert(p.x>=0&&p.x<=w);assert(p.y>=0&&p.y<=h);
    for(let i=0;i<=24;i++)assert(Object.values(koiBody(i/24,p.length,p.phase)).every(Number.isFinite));
  }
});
test('jellyfish keep identifiable pulse and bounded stable bodies',()=>{
  for(const [w,h]of[[420,230],[900,288],[2400,288]])for(const t of[0,1,12,42,120]){
    const a=jellyPoses(w,h,t,9);assert.deepEqual(a,jellyPoses(w,h,t,9));
    for(const p of a){assert(Object.values(p).every(Number.isFinite));assert(p.pulse>=0&&p.pulse<=1);assert(p.radius>0);assert([2,3].includes(p.family));}
  }
  assert.notEqual(jellyPoses(900,288,0,9)[0].pulse,jellyPoses(900,288,1,9)[0].pulse);
});
