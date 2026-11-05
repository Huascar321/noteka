// Persistent logical keys stay compatible. Values are official Icona variants.
export const ICONA_REVISION = 'dd9f5217720a3f459a1f5464125701eb0d51c3a2';
export const ICON_SELECTION = {
  contact: 'user-fill', 'circle-user': 'user-fill', 'user-plus': 'user-fill',
  'phone-call': 'headphones-fill', 'arrow-up': 'arrow-up-outline', 'arrow-down': 'arrow-down-outline',
  grip: 'grip-outline', clock: 'clock-outline', shapes: 'layers-fill',
  rocket: 'rocket-fill', forward: 'arrow-right-outline', columns: 'split-view-fill',
  trophy: 'flag-fill', list: 'menu-outline', mic: 'mic-fill',
  pen: 'pen-tool-fill', users: 'user-fill', asterisk: 'sparkles-fill',
  home: 'home-fill', file: 'file-fill', 'file-text': 'file-fill',
  notebook: 'file-fill', task: 'checklist-outline', project: 'kanban-fill',
  tag: 'tag-outline', person: 'user-fill', search: 'search-outline',
  collections: 'layers-fill', inbox: 'inbox-zero-fill', sun: 'sun-fill',
  archive: 'inbox-zero-fill', circle: 'loader-outline', alert: 'alert-fill',
  edit: 'edit-outline', calendar: 'calendar-fill', folder: 'folder-fill',
  star: 'star-fill', heart: 'heart-fill', book: 'bookmark-fill',
  target: 'target-outline', code: 'code-outline', audio: 'volume-outline',
  camera: 'camera-fill', palette: 'palette-fill', globe: 'compass-fill',
  leaf: 'leaf-fill', lightbulb: 'lightbulb-fill', coffee: 'coffee-fill',
  sparkles: 'sparkles-fill', plane: 'paper-plane-fill', briefcase: 'kanban-fill',
  graduation: 'lightbulb-fill', dumbbell: 'zap-fill', gamepad: 'dice-fill',
  wrench: 'settings-outline', cloud: 'cloud-fill', menu: 'sidebar-collapse-fill',
  key: 'key-outline', 'circle-check': 'check-circle-fill',
  'calendar-week': 'calendar-fill', 'calendar-clock': 'snooze-fill',
  'circle-dot': 'focus-ring-fill', hammer: 'edit-outline', 'list-checks': 'checklist-outline',
  refresh: 'refresh-outline', sliders: 'filter-fill', expand: 'resize-handle-outline',
  plus: 'plus-outline', close: 'x-outline', 'arrow-up-right': 'arrow-outline',
  'arrow-right': 'arrow-right-outline', 'arrow-left': 'arrow-left-outline',
  'chevron-down': 'chevron-down-outline', 'chevron-right': 'chevron-right-outline', check: 'check-outline',
  pause: 'pause-fill', settings: 'settings-outline',
  save: 'download-outline', trash: 'trash-fill',
  'sidebar-right': 'sidebar-collapse-fill', 'app-window': 'split-view-fill',
  ellipsis: 'more-outline', undo: 'undo-outline',
  lock: 'lock-fill', 'lock-open': 'lock-fill',
};
// Small adaptations use only the selected official geometry; no outline-to-fill conversion.
export const ICON_ADAPTATIONS = {
  'user-plus': 'user-plus', users: 'users', forward: 'double-arrow',
  'sidebar-right': 'mirror', 'lock-open': 'open-shackle',
};
export const SIDEBAR_MOTION_DURATION_MS = 1000;
export const SIDEBAR_MOTIONS = {
  home: 'jelly', task: 'jelly', 'file-text': 'jelly',
  project: 'jelly', tag: 'jelly', trophy: 'jelly', person: 'jelly',
  search: 'jelly', collections: 'jelly',
};
