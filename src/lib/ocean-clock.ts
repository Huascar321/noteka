export const MAX_OCEAN_FPS=120;
export const MIN_OCEAN_FPS=30;
export class OceanClock {
  private previous:number|null=null;
  private next=0;
  private seconds=0;
  private paintedAt=0;
  private frame=0;
  private interval=1000/MAX_OCEAN_FPS;
  setFps(fps:number) {
    const interval=1000/Math.min(MAX_OCEAN_FPS,Math.max(MIN_OCEAN_FPS,fps));
    if(interval===this.interval)return;
    this.interval=interval;
    if(this.previous!==null)this.next=this.previous+interval;
  }
  suspend() { this.previous=null; }
  tick(now:number,reduced=false) {
    if(reduced) return {seconds:0,delta:0,frame:0};
    if(this.previous===null) { this.previous=now;this.next=now; }
    this.seconds+=Math.min(.1,Math.max(0,(now-this.previous)/1000));this.previous=now;
    if(now+.12<this.next) return null;
    this.next+=(Math.floor((now+.12-this.next)/this.interval)+1)*this.interval;
    const delta=this.seconds-this.paintedAt;this.paintedAt=this.seconds;
    return {seconds:this.seconds,delta,frame:++this.frame};
  }
}
