/** Approved Due motion, shared by interactive transient surfaces. */
export const LIQUID_OPEN_MS = 400;
export const LIQUID_MENU_OPEN_MS = 160;
export const LIQUID_CONTENT_REVEAL_MS = 100;
export const LIQUID_CLOSE_MS = 200;

export function liquidMaterialFrames(source: Keyframe, radius = '14px', drift = -3, overshoot = { x: 1.035, y: 1.025 }): Keyframe[] {
  return [
    { ...source, offset: 0, easing: 'cubic-bezier(.2,.75,.25,1)' },
    { transform: `translate(0, ${drift}px) scale(${overshoot.x}, ${overshoot.y})`, borderRadius: radius === '14px' ? '18px' : radius, offset: .56, easing: 'ease-in-out' },
    { transform: 'translate(0, 0) scale(.994, .985)', borderRadius: radius, offset: .78, easing: 'ease-out' },
    { transform: 'none', borderRadius: radius, offset: 1 },
  ];
}

export function liquidContentFrames(drift = -3, opacity = 1): Keyframe[] {
  return [
    { opacity: 0, transform: `translateY(${drift * 2}px)`, offset: 0, easing: 'ease-out' },
    { opacity, transform: 'none', offset: LIQUID_CONTENT_REVEAL_MS / LIQUID_OPEN_MS },
    { opacity, transform: 'none', offset: 1 },
  ];
}
