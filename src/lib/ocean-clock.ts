export const MAX_OCEAN_FPS=120;
const INTERVAL=1000/MAX_OCEAN_FPS;
export class OceanClock {
  private previous:number|null=null;
  private next=0;
  private seconds=0;
  private paintedAt=0;
  private frame=0;
  suspend() { this.previous=null; }
  tick(now:number,reduced=false) {
    if(reduced) return {seconds:0,delta:0,frame:0};
    if(this.previous===null) { this.previous=now;this.next=now; }
    this.seconds+=Math.min(.1,Math.max(0,(now-this.previous)/1000));this.previous=now;
    if(now+.12<this.next) return null;
    this.next+=(Math.floor((now+.12-this.next)/INTERVAL)+1)*INTERVAL;
    const delta=this.seconds-this.paintedAt;this.paintedAt=this.seconds;
    return {seconds:this.seconds,delta,frame:++this.frame};
  }
}
