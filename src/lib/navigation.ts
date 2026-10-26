import type { ContextLink, EntityKind, SearchHit, WorkflowView } from './contracts.generated';
import type { IconName } from './icon-set';

export type Section = EntityKind | 'goals' | 'home' | 'archive' | Exclude<WorkflowView, 'context'> | 'areas' | 'resources' | 'search' | 'collections';
export type SectionInfo = { id: Section; label: string; icon: IconName; description: string; aliases: string };

// Route metadata also describes screens that are intentionally absent from the sidebar.
export const SECTIONS: SectionInfo[] = [
  { id: 'home', label: 'Home', icon: 'home', description: "Your workspace", aliases: 'inicio portada' },
  { id: 'task', label: 'Tasks', icon: 'task', description: "All your tasks", aliases: 'tareas' },
  { id: 'note', label: 'Notes', icon: 'file-text', description: "All your notes", aliases: 'notas páginas paginas' },
  { id: 'project', label: 'Projects', icon: 'project', description: "All your projects", aliases: 'proyectos' },
  { id: 'tag', label: 'Tags', icon: 'tag', description: "Tags, areas and resources", aliases: 'etiquetas' },
  { id: 'goals', label: 'Goals', icon: 'trophy', description: "Goals and milestones", aliases: 'goals objetivos metas hitos milestones' },
  { id: 'person', label: 'People', icon: 'person', description: "People and relations", aliases: 'personas contactos' },
  { id: 'collections', label: 'Collections', icon: 'collections', description: "Collections and saved views", aliases: 'collection colecciones base datos tablas' },
  { id: 'search', label: 'Advanced search', icon: 'search', description: "Search notes, attachments or both", aliases: 'search buscar búsqueda busqueda adjuntos attachments' },
  { id: 'inbox', label: 'Inbox', icon: 'inbox', description: "Incoming notes and tasks", aliases: 'bandeja entrada' },
  { id: 'my_day', label: 'My Day', icon: 'sun', description: "Tasks planned for today", aliases: 'mi día dia hoy planificación' },
  { id: 'pending', label: 'Pending', icon: 'circle', description: "All incomplete tasks", aliases: 'pendientes' },
  { id: 'overdue', label: 'Overdue', icon: 'alert', description: "Tasks due before today", aliases: 'vencidas atrasadas' },
  { id: 'active_projects', label: 'Active Projects', icon: 'project', description: "Planned or active projects", aliases: 'proyectos activos' },
  { id: 'areas', label: 'Areas', icon: 'tag', description: "Tags classified as areas", aliases: 'áreas areas' },
  { id: 'resources', label: 'Resources', icon: 'tag', description: "Tags classified as resources", aliases: 'recursos' },
  { id: 'archive', label: 'Archive', icon: 'archive', description: "Archived entities and recovery", aliases: 'archives archivo archivados papelera' },
];

export const PRIMARY_SECTIONS = SECTIONS.slice(0, 7);
export function entitySection(kind: EntityKind, payload?: unknown): Section {
  if (kind === 'note' && payload && typeof payload === 'object' && (('goal' in payload && payload.goal !== null && typeof payload.goal === 'object') || ('milestone' in payload && payload.milestone !== null && typeof payload.milestone === 'object'))) return 'goals';
  return kind;
}
export function newPageLabel(kind: EntityKind, payload?: Record<string, unknown>): string {
  if (kind === 'note' && payload?.goal) return "New Goal";
  if (kind === 'note' && payload?.milestone) return "New Milestone";
  return ({ note: "New note", task: "New task", project: "New project", tag: "New tag", person: "New person" })[kind];
}
export function sectionInfo(id: Section): SectionInfo {
  return SECTIONS.find(section => section.id === id)!;
}

export type NavigationTarget =
  | { type: 'section'; section: Section }
  | { type: 'entity'; id: string; kind: EntityKind; title: string; hit?: SearchHit }
  | { type: 'draft'; kind: EntityKind; context?: ContextLink; seed?: Record<string, unknown> };

export type PaneState = {
  entityId?: string;
  sectionPage?: boolean;
  section: Section;
  title: string;
  icon: string;
  color?: string;
  dirty: boolean;
  busy: boolean;
  guarded: boolean;
};

export type WorkspacePaneApi = {
  navigate: (section: Section) => Promise<boolean>;
  openTarget: (target: NavigationTarget) => Promise<boolean>;
  prepareToLeave: () => Promise<boolean>;
};

export function targetTitle(target: NavigationTarget): string {
  return target.type === 'section' ? sectionInfo(target.section).label : target.type === 'draft' ? newPageLabel(target.kind, target.seed) : target.title;
}

export function targetIcon(target: NavigationTarget): string {
  return sectionInfo(target.type === 'section' ? target.section : target.kind).icon;
}

export function matchingSections(query: string): SectionInfo[] {
  const normalize = (value: string) => value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
  const terms = normalize(query.trim()).split(/\s+/).filter(Boolean);
  return terms.length ? SECTIONS.filter(section => terms.every(term => normalize(`${section.label} ${section.aliases}`).includes(term))) : PRIMARY_SECTIONS;
}
