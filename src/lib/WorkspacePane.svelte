<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { operate, OperationError, pickGoogleDrive, openExternal, googlePreview, type GooglePreview } from './api';
  import { RefreshCoordinator } from './refresh-coordinator';
  import BlockEditor from './BlockEditor.svelte';
  import ScrollOutline from './ScrollOutline.svelte';
  import PeekModeMenu from './PeekModeMenu.svelte';
  import type { PeekMode, PeekPresentation } from './peek-mode';
  import EntityFields from './EntityFields.svelte';
  import TaskPageProperties from './TaskPageProperties.svelte';
  import NotePageProperties from './NotePageProperties.svelte';
  import ProjectPageProperties from './ProjectPageProperties.svelte';
  import ProjectContents from './ProjectContents.svelte';
  import TagPageProperties from './TagPageProperties.svelte';
  import TagContents from './TagContents.svelte';
  import GoalPageProperties from './GoalPageProperties.svelte';
  import PersonPhoto from './PersonPhoto.svelte';
  import PersonContents from './PersonContents.svelte';
  import PageTools from './PageTools.svelte';
  import DatabasePanel from './DatabasePanel.svelte';
  import HomePanel from './HomePanel.svelte';
  import TasksPage from './TasksPage.svelte';
  import NotesPage from './NotesPage.svelte';
  import ProjectsPage from './ProjectsPage.svelte';
  import TagsPage from './TagsPage.svelte';
  import GoalsPage from './GoalsPage.svelte';
  import PeoplePage from './PeoplePage.svelte';
  import MilestonesPanel from './MilestonesPanel.svelte';
  import GoalProjects from './GoalProjects.svelte';
  import { splitGoalDocument, composeGoalDocument, goalHasContent } from './goal-document';
  import PageBreadcrumb from './PageBreadcrumb.svelte';
  import AppIcon from './AppIcon.svelte';
  import IconPicker from './IconPicker.svelte';
  import RelationPicker from './RelationPicker.svelte';
  import { entityIcon, entityIconColor } from './entity-appearance';
  import { importedPresentation, composeImportedBody, propertyStyle, type ImportedPresentation } from './imported-properties';
  import { sectionInfo, entitySection, newPageLabel, type Section, type NavigationTarget, type PaneState } from './navigation';
  import AttachmentSearchPanel from './AttachmentSearchPanel.svelte';
  import AttachmentPreparationPanel from './AttachmentPreparationPanel.svelte';
  import AttachmentGenerationPanel from './AttachmentGenerationPanel.svelte';
  import GroupedSearchPanel from './GroupedSearchPanel.svelte';
  import { pendingJson } from './editor-adapter';
  import type { Entity, EntityKind, WorkspaceInfo, Document, Contract, Relation, ContextLink, WorkflowView, SearchHit, ExternalResource, HomeSummary } from './contracts.generated';
  type PageEntity = Contract['entities'];
  type PageRelation = Contract['relations'];

  type Draft = { kind: EntityKind; title: string; payload: Record<string, unknown>; document?: Document; context?: ContextLink };
  type Status = 'saved' | 'dirty' | 'saving' | 'error' | 'conflict';
  type Guard = { action: () => Promise<void>; resolve: (allowed: boolean) => void };
  let { workspace, paneId, active, visible = active, sessionActive = active, animateAlongsideSidePeek = false, blocked = false, searchOpen = false, initialTarget, onstate, onrequestdetail, onnavigatesection, peekMode = 'full', defaultPeekMode = 'side', peekPreferenceError = '', onpresent, ondefaultpeek, onclosepeek }: {
    workspace: WorkspaceInfo; paneId: string; active: boolean; blocked?: boolean; searchOpen?: boolean;
    visible?: boolean; sessionActive?: boolean; animateAlongsideSidePeek?: boolean; peekMode?: PeekPresentation; defaultPeekMode?: PeekMode; peekPreferenceError?: string;
    onrequestdetail?: (target: NavigationTarget) => Promise<void>;
    onnavigatesection?: (section: Section) => Promise<void>;
    onpresent?: (mode: PeekMode) => void; ondefaultpeek?: (mode: PeekMode) => void; onclosepeek?: () => void;
    initialTarget: NavigationTarget; onstate: (id: string, state: PaneState) => void;
  } = $props();
  let section = $state<Section>(untrack(() => initialTarget.type === 'section' ? initialTarget.section : initialTarget.kind));
  let viewMode = $state<'overview' | 'detail'>(untrack(() => initialTarget.type !== 'section' ? 'detail' : 'overview'));
  let ready = $state(false);
  let disposed = false;
  let lastActive = false;
  let entities = $state<Entity[]>([]);
  let nextCursor = $state<string | null>(null);
  let selected = $state<Entity | null>(null);
  let draft = $state<Draft | null>(null);
  let status = $state<Status>('saved');
  let error = $state('');
  let listError = $state('');
  let loading = $state(false);
  let listGeneration = 0;
  let listController: AbortController | undefined;
  let listIdentity = '';
  let listHasResult = false;
  let listPages = 0;
  let listStale = $state(false);
  let listNotice = $state('');
  let working = $state(false);
  let remoteRevision = $state<number | null>(null);
  let relationships = $state<{ relation: Relation; other: Entity }[]>([]);
  let relationReadGeneration = 0;
  let relationReadController: AbortController | undefined;
  let dashboardReadController: AbortController | undefined;
  let detailRefreshController: AbortController | undefined;
  let externalResources = $state<ExternalResource[]>([]);
  let externalError = $state('');
  let externalBusy = $state(false);
  let externalPreview = $state<GooglePreview | null>(null);
  let externalPreviewError = $state('');
  let externalPreviewBusy = $state(false);
  let guard = $state<Guard | null>(null);
  let recovery = $state('');
  let titleInput = $state<HTMLInputElement | HTMLTextAreaElement>();
  let editorRoot = $state<HTMLElement>();
  let editorEpoch = $state(0);
  let editorInitial: Document | null = $state(null);
  let goalOverviewInitial: Document | null = $state(null);
  let goalJournalInitial: Document | null = $state(null);
  let goalDetailsInitial: Document | null = $state(null);
  let goalDetailsOpen = $state(false);
  let goalLegacyInitial: Document | null = $state(null);
  let goalLegacyVisible = $state(false);
  let goalMarkerId = '';
  let goalInvalidSections: Record<string, string> = {};
  let goalJournalEditor = $state<{ appendGoalLog: (date: string) => void }>();
  let bodyEditor = $state<{ focusStart: () => void }>();
  let importInfo = $state<ImportedPresentation | null>(null);
  let detailsOpen = $state(false);
  let utilityPanel = $state<string | null>(null);
  const typedGoalPage = $derived(draft?.kind === 'note' && !!(draft.payload.goal || draft.payload.milestone));
  const plainNotePage = $derived(draft?.kind === 'note' && !typedGoalPage);
  let temporaryUnlockFor = $state<string | null>(null);
  let lockStatusButton = $state<HTMLButtonElement>();
  const pageLocked = $derived(plainNotePage && (selected?.payload as Record<string, unknown> | undefined)?.page_locked === true);
  const noteReadOnly = $derived(pageLocked && temporaryUnlockFor !== selected?.id);
  const noteEditingDisabled = $derived(working || status === 'saving' || noteReadOnly);
  $effect(() => { if (!pageLocked) temporaryUnlockFor = null; });
  $effect(() => { if (!visible || guard || blocked) relationPicker = null; });
  const primaryImported = $derived(plainNotePage ? [] : importInfo?.properties.filter(property => !['Created','Updated','Archived','Created By','Edited By','Edited', ...(typedGoalPage ? ['Status','Goal Set','Target Deadline','Achieved','Progress','Latest Activity','Projects','Tag','Milestones','Goal','Goal Area','Date Completed','This Quarter','This Year'] : [])].includes(property.name)) ?? []);
  let exportPath = $state('');
  let exportMessage = $state('');
  let invalidEditor = $state('');
  let workflowDate = $state('');
  let dashboard = $state<Entity[]>([]);
  let dashboardCursor = $state<string | null>(null);
  let dashboardBusy = $state(false);
  let dashboardError = $state('');
  let dashboardStale = $state(false);
  let dashboardNotice = $state('');
  let dashboardHasResult = false;
  let dashboardPages = 0;
  let dashboardGeneration = 0;
  let projectSummary = $state<Extract<HomeSummary, { kind: 'project' }> | null>(null);
  let projectSummaryBusy = $state(false);
  let projectSummaryError = $state('');
  let projectSummaryGeneration = 0;
  let tagSummary = $state<Extract<HomeSummary, { kind: 'tag' }> | null>(null);
  let tagSummaryBusy = $state(false);
  let tagSummaryError = $state('');
  let tagSummaryGeneration = 0;
  let goalSummary = $state<Extract<HomeSummary, { kind: 'goal' }> | null>(null);
  let goalSummaryBusy = $state(false);
  let goalSummaryError = $state('');
  let goalSummaryGeneration = 0;
  let relationPicker = $state<{ kind: 'project' | 'tag' | 'person'; anchor: HTMLElement } | null>(null);
  let relationError = $state('');
  let searchQuery = $state('');
  let searchScope = $state<'notes' | 'attachments' | 'combined'>('notes');
  let groupedPanel = $state<{ refreshPanel: () => Promise<void> }>();
  let attachmentPanel = $state<{ refreshPanel: () => Promise<void> }>();
  let searchKind = $state('');
  let searchArchived = $state(false);
  let searchRelated = $state('');
  let searchHits = $state<SearchHit[]>([]);
  let searchMessage = $state('');
  let sourceLocator = $state('');
  let sourceBlock = $state<string | undefined>();
  const createKind = $derived(['note', 'task', 'project', 'tag', 'person'].includes(section) ? section as EntityKind : section === 'areas' || section === 'resources' ? 'tag' : section === 'my_day' || section === 'pending' || section === 'overdue' ? 'task' : section === 'active_projects' ? 'project' : 'note');
  const createLabel: Record<EntityKind, string> = { note: "New note", task: "New task", project: "New project", tag: "New tag", person: "New person" };
  function sectionIcon() { return sectionInfo(section).icon; }
  function chooseIcon(value: { icon: string; color: string | null }) {
    if (!draft || noteReadOnly) return;
    if (value.icon) draft.payload.icon = value.icon;
    if (value.color) draft.payload.icon_color = value.color; else delete draft.payload.icon_color;
    edited();
  }
  let collectionPending = $state(false);
  let homePanel = $state<{ refreshPanel: () => Promise<void> }>();
  let tasksPage = $state<{ refreshPanel: () => Promise<void> }>();
  let notesPage = $state<{ refreshPanel: () => Promise<void> }>();
  let projectsPage = $state<{ refreshPanel: () => Promise<void> }>();
  let tagsPage = $state<{ refreshPanel: () => Promise<void> }>();
  let goalsPage = $state<{ refreshPanel: () => Promise<void> }>();
  let peoplePage = $state<{ refreshPanel: () => Promise<void> }>();
  let goalMilestones = $state<{ refreshPanel: () => Promise<void> }>();
  let databasePanel = $state<{ savePending: () => Promise<boolean>; discardPending: () => void; refreshPanel: () => Promise<void> }>();
  const dirty = $derived((draft !== null && status !== 'saved') || collectionPending);
  const currentLabel = $derived(sectionInfo(section).label);
  const taskProjects = $derived(relationships.filter(item => item.relation.kind === 'project' && item.other.kind === 'project').map(item => item.other));
  const noteTags = $derived(relationships.filter(item => item.relation.kind === 'tag' && item.other.kind === 'tag').map(item => item.other));
  const noteSourceUrl = $derived(importInfo?.properties.find(property => property.name === 'URL')?.value ?? '');
  const pageKind = $derived(draft?.kind ?? selected?.kind);
  const outlineNote = $derived.by(() => { const payload = (draft?.payload ?? selected?.payload ?? {}) as Record<string, unknown>; return pageKind === 'note' && !payload.goal && !payload.milestone; });
  const projectDetail = $derived(pageKind === 'project');
  const tagDetail = $derived(pageKind === 'tag');
  const goalDetail = $derived(draft?.kind === 'note' && !!draft.payload.goal);
  const personDetail = $derived(pageKind === 'person');
  const personPayload = $derived((draft?.payload ?? selected?.payload ?? {}) as Record<string, unknown>);
  const goalProjects = $derived(relationships.filter(item => item.other.kind === 'project').map(item => item.other));
  const toolGroups = $derived([
    ...(pageKind === 'note' || pageKind === 'task' || selected?.document ? [{ id: 'export', label: "Export", icon: 'arrow-up', available: !!selected?.document && !dirty, reason: dirty ? "Save changes to export" : "Requires saved content" }] : []),
    { id: 'resources', label: "External resources", icon: 'globe', available: !!selected, reason: "Save the page to link resources" },
    ...(pageKind === 'note' ? [{ id: 'attachments', label: "Attachments", icon: 'folder', available: true }] : []),
  ]);
  const overview = $derived(viewMode === 'overview' && !['home', 'collections'].includes(section));
  const breadcrumbRecord = $derived(draft ? {
    kind: draft.kind, owner: entitySection(draft.kind,draft.payload), title: draft.title || newPageLabel(draft.kind,draft.payload),
    icon: entityIcon(draft.kind, draft.title, draft.payload), color: entityIconColor(draft.payload),
  } : selected ? {
    kind: selected.kind, owner: entitySection(selected.kind,selected.payload), title: selected.title,
    icon: entityIcon(selected.kind, selected.title, selected.payload), color: entityIconColor(selected.payload),
  } : null);

  const refreshCoordinator = new RefreshCoordinator(
    () => !disposed && ready && !working && !blocked && status !== 'saving',
    () => performRefresh(),
    e => { if (!disposed) error = report(e); },
  );
  $effect(() => {
    const state: PaneState = {
      entityId: selected?.id,
      sectionPage: !draft && !selected,
      section, title: draft ? (draft.title || newPageLabel(draft.kind,draft.payload)) : selected?.title || currentLabel,
      icon: draft ? entityIcon(draft.kind, draft.title, draft.payload) : selected ? entityIcon(selected.kind, selected.title, selected.payload) : sectionIcon(),
      color: entityIconColor(draft?.payload ?? selected?.payload), dirty,
      busy: working || externalBusy || externalPreviewBusy || status === 'saving', guarded: guard !== null,
    };
    untrack(() => onstate(paneId, state));
  });
  $effect(() => {
    const isActive = sessionActive;
    if (isActive && !lastActive) untrack(() => { if (ready) void tick().then(() => { fitTitle(); void refresh(); }); });
    lastActive = isActive;
  });
  $effect(() => {
    const allowed = ready && !working && !blocked && status !== 'saving';
    if (allowed) untrack(() => refreshCoordinator.notifyReady());
  });
  function report(e: unknown) { return e instanceof Error ? e.message : "The operation could not be completed."; }
  async function guarded(action: () => Promise<void>): Promise<boolean> {
    if (disposed || working || externalBusy || status === 'saving' || guard) return false;
    if (dirty) return new Promise(resolve => { guard = { action, resolve }; });
    await action(); return true;
  }
  export async function prepareToLeave(): Promise<boolean> { return guarded(async () => {}); }
  async function resolveGuard(choice: 'save' | 'discard' | 'cancel') {
    const pending = guard;
    if (!pending) return;
    if (choice === 'cancel') { guard = null; pending.resolve(false); return; }
    if (choice === 'save' && !(await save())) return;
    guard = null;
    if (choice === 'discard') { databasePanel?.discardPending(); draft = null; status = 'saved'; }
    try { await pending.action(); pending.resolve(true); }
    catch (e) { error = report(e); pending.resolve(false); }
  }
  function clearSelection() {
    detailRefreshController?.abort(); relationReadController?.abort(); dashboardReadController?.abort();
    temporaryUnlockFor = null;
    relationReadGeneration++;
    relationPicker = null; relationError = '';
    importInfo = null; detailsOpen = false; utilityPanel = null;
    tagSummaryGeneration++; tagSummary = null; tagSummaryBusy = false; tagSummaryError = '';
    goalSummaryGeneration++; goalSummary = null; goalSummaryBusy = false; goalSummaryError = '';
    dashboardGeneration++; projectSummaryGeneration++; dashboardBusy = false; dashboardError = ""; dashboardStale = false; dashboardNotice = ''; dashboardHasResult = false; dashboardPages = 0; projectSummary = null; projectSummaryBusy = false; projectSummaryError = "";
    selected = null; draft = null; status = 'saved'; relationships = []; externalResources = []; externalError = ''; externalBusy = false; externalPreview = null; externalPreviewError = ''; externalPreviewBusy = false; dashboard = []; dashboardCursor = null; remoteRevision = null; error = ''; recovery = ''; exportMessage = ''; sourceLocator = ''; sourceBlock = undefined; invalidEditor = ''; goalInvalidSections = {}; editorInitial = null; goalOverviewInitial = null; goalJournalInitial = null; goalDetailsInitial = null; goalDetailsOpen = false; goalLegacyInitial = null; goalLegacyVisible = false; editorEpoch++;
  }
  function fitTitle() {
    if (!(titleInput instanceof HTMLTextAreaElement)) return;
    titleInput.style.height = 'auto';
    titleInput.style.height = `${titleInput.scrollHeight}px`;
  }
  $effect(() => {
    const root = editorRoot;
    if (!root || !visible) return;
    const resize = new ResizeObserver(fitTitle);
    resize.observe(root);
    return () => resize.disconnect();
  });
  function emptyEditorDocument(): Document {
    return { schema_version: 2, blocks: [{ id: crypto.randomUUID(), type: 'paragraph', content: [] }] };
  }
  function titleKeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return;
    event.preventDefault();
    if (noteReadOnly) return;
    if ((draft?.kind === 'task' || plainNotePage) && !event.metaKey && !event.ctrlKey && !event.altKey) void tick().then(() => bodyEditor?.focusStart());
  }
  function openPageRelations(kind: 'project' | 'tag' | 'person', anchor: HTMLElement) {
    if (noteReadOnly || !selected || working || status === 'saving') return;
    relationError = '';
    relationPicker = { kind, anchor };
  }
  function initializeGoalEditors(body: Document) {
    const parts = splitGoalDocument(body);
    goalMarkerId = parts.marker ?? crypto.randomUUID();
    const segment = (blocks: Document['blocks']): Document => ({ ...body, blocks: blocks.length ? JSON.parse(JSON.stringify(blocks)) as Document['blocks'] : emptyEditorDocument().blocks });
    goalOverviewInitial = segment(parts.overview);
    goalJournalInitial = segment(parts.journal);
    goalDetailsInitial = segment(parts.details);
    goalDetailsOpen = goalHasContent(parts.details);
    goalLegacyInitial = segment(parts.legacy);
    goalLegacyVisible = goalHasContent(parts.legacy);
    goalInvalidSections = {};
  }
  function goalInvalid(section: string, raw: string, message: string) {
    goalInvalidSections[section] = raw;
    invalidEditor = JSON.stringify({ original: draft?.document, invalid_sections: goalInvalidSections }, null, 2);
    error = message; edited();
  }
  function goalChanged(section: 'legacy' | 'overview' | 'details' | 'journal', changed: Document) {
    if (!draft?.document || noteReadOnly) return;
    delete goalInvalidSections[section];
    const prefix = importInfo?.prefix ?? [];
    const body: Document = { ...draft.document, blocks: draft.document.blocks.slice(prefix.length) };
    const parts = splitGoalDocument(body);
    const version = Math.max(body.schema_version, changed.schema_version);
    if (!parts.marker && section === 'legacy') {
      draft.document = { ...body, schema_version: version, blocks: [...prefix, ...changed.blocks] };
    } else {
      if (!parts.marker) {
        parts.overview = JSON.parse(JSON.stringify(goalOverviewInitial?.blocks ?? emptyEditorDocument().blocks));
        parts.journal = JSON.parse(JSON.stringify(goalJournalInitial?.blocks ?? emptyEditorDocument().blocks));
        parts.details = JSON.parse(JSON.stringify(goalDetailsInitial?.blocks ?? emptyEditorDocument().blocks));
      }
      parts[section] = changed.blocks;
      const composed = composeGoalDocument({ ...body, schema_version: version }, parts, parts.marker ?? goalMarkerId);
      draft.document = { ...composed, blocks: [...prefix, ...composed.blocks] };
    }
    invalidEditor = Object.keys(goalInvalidSections).length ? JSON.stringify({ original: draft.document, invalid_sections: goalInvalidSections }, null, 2) : '';
    edited();
  }
  function addGoalLog() {
    const parts = new Intl.DateTimeFormat('en-CA', { timeZone: workspace.timezone, year: 'numeric', month: '2-digit', day: '2-digit' }).formatToParts(new Date());
    const value = (type: string) => parts.find(part => part.type === type)?.value ?? '';
    goalJournalEditor?.appendGoalLog(`${value('year')}-${value('month')}-${value('day')}`);
  }
  function loadDraft(entity: Entity) {
    const validPayload = entity.payload !== null && typeof entity.payload === 'object' && !Array.isArray(entity.payload);
    const hasBody = entity.kind === 'note' || entity.kind === 'task';
    draft = validPayload && (!hasBody || entity.document_editable) ? { kind: entity.kind, title: entity.title, payload: JSON.parse(JSON.stringify(entity.payload)) as Record<string, unknown>, ...(hasBody && entity.document ? { document: JSON.parse(JSON.stringify(entity.document)) as Document } : {}) } : null;
    const source = draft?.payload.source as { provider?: string; collection?: string } | undefined;
    const presentation = draft?.document ? draft.kind === 'note' ? importedPresentation(draft.document, source?.provider === 'notion' && (draft?.payload.goal || draft?.payload.milestone || source.collection === 'Notes') ? source.collection : undefined) : { properties: [], prefix: [], body: draft.document } : null;
    importInfo = presentation?.properties.length ? presentation : null;
    editorInitial = presentation ? JSON.parse(JSON.stringify(presentation.body)) as Document : draft?.kind === 'task' ? emptyEditorDocument() : null;
    if (draft?.kind === 'note' && draft.payload.goal && editorInitial) initializeGoalEditors(editorInitial);
    editorEpoch++;
  }
  async function loadList(cursor?: string) {
    if (!workspace) return;
    if (cursor && (loading || listStale)) return;
    const workspacePath = workspace.path;
    const requestedSection = section;
    const generation = ++listGeneration;
    listController?.abort();
    const controller = listController = new AbortController();
    const identity = JSON.stringify(requestedSection === 'search'
      ? [workspacePath, requestedSection, searchScope, searchQuery.trim(), searchKind, searchArchived, searchRelated.trim()]
      : [workspacePath, requestedSection]);
    const same = identity === listIdentity;
    const previousPages = same ? listPages : 0;
    if (!same) {
      entities = []; searchHits = []; nextCursor = null; workflowDate = '';
      listIdentity = identity; listHasResult = false; listPages = 0; listNotice = '';
    }
    if (cursor) listNotice = '';
    listStale = same && listHasResult && !cursor;
    loading = true; listError = '';
    try {
      if (requestedSection === 'collections' || requestedSection === 'home' || requestedSection === 'task' || requestedSection === 'note' || requestedSection === 'project' || requestedSection === 'tag' || requestedSection === 'goals' || requestedSection === 'person') { entities = []; nextCursor = null; workflowDate = ''; listStale = false; listHasResult = false; listPages = 0; return; }
      if (requestedSection === 'search') {
        if (searchScope !== 'notes') { entities = []; nextCursor = null; workflowDate = ''; listStale = false; return; }
        const page = searchQuery.trim() ? await operate<Contract['search']>({ operation: 'search.query', input: { query: searchQuery.trim(), kind: searchKind ? searchKind as EntityKind : undefined, archived: searchArchived, related_id: searchRelated.trim() || undefined, limit: 30, cursor } }, workspacePath, { signal: controller.signal }) : { items: [], next_cursor: null };
        if (generation !== listGeneration || workspace?.path !== workspacePath || section !== requestedSection) return;
        searchHits = cursor ? [...searchHits, ...page.items] : page.items; entities = []; nextCursor = page.next_cursor ?? null; workflowDate = '';
        listHasResult = true; listPages = cursor ? previousPages + 1 : 1; listStale = false;
        if (!cursor && previousPages > 1) listNotice = "Additional pages were reset. Load more if available.";
        return;
      }
      const workflow = ['inbox', 'my_day', 'pending', 'overdue', 'active_projects'].includes(requestedSection);
      const page = workflow
        ? await operate<Contract['workflow']>({ operation: 'workflow.query', input: { view: requestedSection as WorkflowView, cursor, limit: 50 } }, workspacePath, { signal: controller.signal })
        : await operate<PageEntity>({ operation: 'entity.list', input: { archived: requestedSection === 'archive', ...(requestedSection !== 'archive' ? { kind: requestedSection === 'areas' || requestedSection === 'resources' ? 'tag' : requestedSection as EntityKind } : {}), ...(requestedSection === 'areas' || requestedSection === 'resources' ? { classification: requestedSection === 'areas' ? 'area' : 'resource' } : {}), cursor, limit: 50 } }, workspacePath, { signal: controller.signal });
      if (generation !== listGeneration || workspace?.path !== workspacePath || section !== requestedSection) return;
      entities = cursor ? [...entities, ...page.items] : page.items; nextCursor = page.next_cursor ?? null;
      workflowDate = 'date' in page ? page.date as string : '';
      listHasResult = true; listPages = cursor ? previousPages + 1 : 1; listStale = false;
      if (!cursor && previousPages > 1) listNotice = "Additional pages were reset. Load more if available.";
    } catch (e) { if (generation === listGeneration && workspace?.path === workspacePath && section === requestedSection) { listError = report(e); listStale = listHasResult; } }
    finally { if (generation === listGeneration) { loading = false; if (listController === controller) listController = undefined; } }
  }
  export async function navigate(to: Section): Promise<boolean> {
    if (onnavigatesection) { await onnavigatesection(to); return true; }
    return guarded(async () => { viewMode = 'overview'; section = to; clearSelection(); await loadList(); });
  }
  function changeSearchScope(scope: 'notes' | 'attachments' | 'combined') {
    listGeneration++; loading = false; listError = ''; nextCursor = null; searchHits = []; searchMessage = '';
    searchScope = scope;
    void loadList();
  }
  function invalidateSearchCriteria() {
    listGeneration++; listController?.abort(); loading = false; listError = '';
    entities = []; searchHits = []; nextCursor = null; workflowDate = '';
    listIdentity = ''; listHasResult = false; listPages = 0; listStale = false; listNotice = '';
  }
  async function openSearch(hit: SearchHit) {
    if (onrequestdetail) { await onrequestdetail({ type: 'entity', id: hit.entity_id, kind: hit.kind, title: hit.title, hit }); return; }
    await guarded(async () => {
      if (!workspace) return;
      viewMode = 'detail'; working = true;
      try {
        await openOriginal(hit.entity_id);
        if (selected?.id !== hit.entity_id) return;
        sourceLocator = `Source: ${hit.entity_id} · block ${hit.block_id ?? "title"} · result r${hit.revision}${selected.revision !== hit.revision ? ` · current source r${selected.revision}; search again` : ''}`;
        sourceBlock = hit.block_id && selected.revision === hit.revision ? hit.block_id : undefined;
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function rebuildSearch() {
    if (!workspace || working) return;
    const workspacePath = workspace.path;
    working = true; searchMessage = '';
    try {
      const index = await operate<Contract['search_index']>({ operation: 'search.rebuild', input: {} }, workspacePath);
      if (workspace?.path !== workspacePath) return;
      searchMessage = `Index rebuilt: ${index.entities} entities, ${index.chunks} sources. Originals unchanged.`;
      await loadList();
    } catch (e) { listError = report(e); }
    finally { working = false; }
  }
  async function loadRelations(entity: Entity) {
    if (!workspace) return;
    const workspacePath = workspace.path;
    const generation = ++relationReadGeneration;
    relationReadController?.abort();
    const controller = relationReadController = new AbortController();
    try {
      const all: Relation[] = [];
      let cursor: string | null | undefined;
      do {
        const page = await operate<PageRelation>({ operation: 'relation.list', input: { entity_id: entity.id, cursor, limit: 200 } }, workspacePath, { signal: controller.signal });
        all.push(...page.items); cursor = page.next_cursor;
      } while (cursor);
      const resolved: { relation: Relation; other: Entity }[] = new Array(all.length);
      let next = 0;
      await Promise.all(Array.from({ length: Math.min(2, all.length) }, async () => {
        while (next < all.length) {
          const index = next++, relation = all[index];
          const other = await operate<Entity>({ operation: 'entity.get', input: { id: relation.source_id === entity.id ? relation.target_id : relation.source_id } }, workspacePath, { signal: controller.signal });
          resolved[index] = { relation, other };
        }
      }));
      if (generation === relationReadGeneration && workspace?.path === workspacePath && selected?.id === entity.id) relationships = resolved;
    } catch (e) {
      if (!controller.signal.aborted && generation === relationReadGeneration) throw e;
    } finally { if (relationReadController === controller) relationReadController = undefined; }
  }
  async function loadProjectSummary(entity: Entity, signal?: AbortSignal) {
    if (entity.kind !== 'project') return;
    const path = workspace.path, generation = ++projectSummaryGeneration;
    projectSummaryBusy = true; projectSummaryError = '';
    try {
      const page = await operate<Contract['home']>({ operation: 'home.query', input: { view: 'projects_board', limit: 100 } }, path, { signal });
      if (workspace.path !== path || selected?.id !== entity.id || generation !== projectSummaryGeneration) return;
      const summary = page.items.find(item => item.entity.id === entity.id)?.summary;
      projectSummary = summary?.kind === 'project' ? summary : null;
    } catch (e) { if (!signal?.aborted && workspace.path === path && selected?.id === entity.id && generation === projectSummaryGeneration) { projectSummary = null; projectSummaryError = report(e); } }
    finally { if (workspace.path === path && selected?.id === entity.id && generation === projectSummaryGeneration) projectSummaryBusy = false; }
  }
  async function loadGoalSummary(entity: Entity, signal?: AbortSignal) {
    const path = workspace.path, generation = ++goalSummaryGeneration;
    goalSummaryBusy = true; goalSummaryError = '';
    try {
      const page = await operate<Contract['home']>({ operation: 'home.query', input: { view: 'goals_activity', limit: 100 } }, path, { signal });
      if (workspace.path !== path || selected?.id !== entity.id || generation !== goalSummaryGeneration) return;
      const summary = page.items.find(item => item.entity.id === entity.id)?.summary;
      goalSummary = summary?.kind === 'goal' ? summary : null;
    } catch (e) { if (!signal?.aborted && workspace.path === path && selected?.id === entity.id && generation === goalSummaryGeneration) { goalSummary = null; goalSummaryError = report(e); } }
    finally { if (workspace.path === path && selected?.id === entity.id && generation === goalSummaryGeneration) goalSummaryBusy = false; }
  }
  function focusTagNotes() {
    const heading = editorRoot?.querySelector<HTMLElement>('[data-tag-notes] h2');
    heading?.focus(); heading?.scrollIntoView({ block: 'start' });
  }
  async function loadTagSummary(entity: Entity, signal?: AbortSignal) {
    if (entity.kind !== 'tag') return;
    const path = workspace.path, generation = ++tagSummaryGeneration;
    tagSummaryBusy = true; tagSummaryError = '';
    try {
      const page = await operate<Contract['home']>({ operation: 'home.query', input: { view: 'tags_alphabetical', limit: 100 } }, path, { signal });
      if (workspace.path !== path || selected?.id !== entity.id || generation !== tagSummaryGeneration) return;
      const summary = page.items.find(item => item.entity.id === entity.id)?.summary;
      tagSummary = summary?.kind === 'tag' ? summary : null;
    } catch (e) { if (!signal?.aborted && workspace.path === path && selected?.id === entity.id && generation === tagSummaryGeneration) { tagSummary = null; tagSummaryError = report(e); } }
    finally { if (workspace.path === path && selected?.id === entity.id && generation === tagSummaryGeneration) tagSummaryBusy = false; }
  }
  async function loadDashboard(entity: Entity, cursor?: string) {
    if (!workspace) return;
    if (cursor && (dashboardBusy || dashboardStale)) return;
    const goal = entity.kind === 'note' && (entity.payload as Record<string, unknown>)?.goal;
    if (!goal && !['project', 'tag', 'person'].includes(entity.kind)) return;
    dashboardReadController?.abort();
    const controller = dashboardReadController = new AbortController();
    if (goal) {
      try { await loadGoalSummary(entity, controller.signal); }
      finally { if (dashboardReadController === controller) dashboardReadController = undefined; }
      return;
    }
    const workspacePath = workspace.path, generation = ++dashboardGeneration;
    const previousPages = dashboardPages;
    dashboardStale = dashboardHasResult && !cursor;
    if (cursor) dashboardNotice = '';
    const summaryRead = !cursor ? entity.kind === 'project' ? loadProjectSummary(entity, controller.signal) : entity.kind === 'tag' ? loadTagSummary(entity, controller.signal) : Promise.resolve() : Promise.resolve();
    dashboardBusy = true; dashboardError = '';
    try {
      const page = await operate<Contract['workflow']>({ operation: 'workflow.query', input: { view: 'context', related_id: entity.id, limit: 50, cursor } }, workspacePath, { signal: controller.signal });
      if (selected?.id === entity.id && workspace.path === workspacePath && generation === dashboardGeneration) {
        dashboard = cursor ? [...dashboard, ...page.items.filter(item => !dashboard.some(previous => previous.id === item.id))] : page.items; dashboardCursor = page.next_cursor ?? null;
        dashboardHasResult = true; dashboardPages = cursor ? previousPages + 1 : 1; dashboardStale = false;
        if (!cursor && previousPages > 1) dashboardNotice = "Additional pages were reset. Load more if available.";
      }
    } catch (e) {
      if (!controller.signal.aborted && selected?.id === entity.id && workspace.path === workspacePath && generation === dashboardGeneration) { dashboardError = report(e); dashboardStale = dashboardHasResult; }
    } finally { if (selected?.id === entity.id && workspace.path === workspacePath && generation === dashboardGeneration) dashboardBusy = false; }
    await summaryRead;
    if (dashboardReadController === controller) dashboardReadController = undefined;
  }
  async function loadExternal(entity: Entity) {
    if (!workspace) return;
    const workspacePath = workspace.path;
    try {
      const page = await operate<Contract['external_resources']>({ operation: 'external.list', input: { entity_id: entity.id, limit: 50 } }, workspacePath);
      if (selected?.id === entity.id && workspace?.path === workspacePath) { externalResources = page.items; externalError = ''; }
    } catch (e) {
      if (selected?.id === entity.id && workspace?.path === workspacePath) externalError = report(e);
    }
  }
  function externalMetadata(resource: ExternalResource): Record<string, unknown> {
    return resource.metadata !== null && typeof resource.metadata === 'object' && !Array.isArray(resource.metadata) ? resource.metadata as Record<string, unknown> : {};
  }
  function externalTitle(resource: ExternalResource): string {
    const name = externalMetadata(resource).name;
    return typeof name === 'string' && name.trim() ? name : resource.resource_id;
  }
  function externalLink(resource: ExternalResource): string | null {
    const link = externalMetadata(resource).webViewLink;
    return typeof link === 'string' && link.startsWith('https://') ? link : null;
  }
  function externalMime(resource: ExternalResource): string {
    const mime = externalMetadata(resource).mimeType;
    return typeof mime === 'string' ? mime : '';
  }
  function externalCanPreview(resource: ExternalResource): boolean {
    return ['application/vnd.google-apps.document', 'application/vnd.google-apps.spreadsheet'].includes(externalMime(resource));
  }
  async function previewResource(resource: ExternalResource) {
    if (!selected || externalPreviewBusy) return;
    externalPreviewBusy = true; externalPreviewError = ''; externalPreview = null;
    try { externalPreview = await googlePreview(resource.resource_id); }
    catch (e) { externalPreviewError = report(e); }
    finally { externalPreviewBusy = false; }
  }
  function closePreview() { externalPreview = null; externalPreviewError = ''; }
  async function linkGoogleDrive() {
    if (!workspace || !selected || dirty || working || externalBusy || noteReadOnly) return;
    const entityId = selected.id;
    const workspacePath = workspace.path;
    externalBusy = true; externalError = '';
    try {
      const picked = await pickGoogleDrive();
      if (!picked.files.length) { externalError = "Google Picker returned no files."; return; }
      let revision = selected.revision;
      for (const file of picked.files) {
        const result = await operate<Contract['external_resource_result']>({
          operation: 'external.link',
          input: {
            entity_id: entityId,
            expected_revision: revision,
            provider: 'google_drive',
            account_id: picked.account_id,
            resource_type: 'drive_file',
            resource_id: file.id,
            metadata: {
              name: file.name,
              mimeType: file.mimeType,
              modifiedTime: file.modifiedTime,
              webViewLink: file.webViewLink,
            },
          },
        }, workspacePath);
        revision = result.entity.revision;
        if (selected?.id === entityId && workspace?.path === workspacePath) selected = result.entity;
      }
      if (selected?.id === entityId) await loadExternal(selected);
    } catch (e) {
      externalError = report(e);
      if (selected?.id === entityId && workspace?.path === workspacePath) {
        try { selected = await operate<Entity>({ operation: 'entity.get', input: { id: entityId } }, workspacePath); await loadExternal(selected); } catch { /* conservar error original */ }
      }
    } finally { externalBusy = false; }
  }
  async function unlinkExternal(resource: ExternalResource) {
    if (!workspace || !selected || dirty || working || externalBusy || noteReadOnly) return;
    const entityId = selected.id;
    const workspacePath = workspace.path;
    externalBusy = true; externalError = '';
    try {
      const result = await operate<Contract['external_resource_result']>({ operation: 'external.unlink', input: { entity_id: entityId, expected_revision: selected.revision, id: resource.id } }, workspacePath);
      if (selected?.id === entityId && workspace?.path === workspacePath) { selected = result.entity; await loadExternal(selected); }
    } catch (e) { externalError = report(e); }
    finally { externalBusy = false; }
  }
  async function openResource(resource: ExternalResource) {
    const link = externalLink(resource);
    if (!link) return;
    try { await openExternal(link); } catch (e) { externalError = report(e); }
  }
  async function openEntity(entity: Entity) {
    if (onrequestdetail) { await onrequestdetail({ type: 'entity', id: entity.id, kind: entity.kind, title: entity.title }); return; }
    await guarded(() => openOriginal(entity.id));
  }
  async function openOriginal(id: string) {
      if (!workspace) return;
      viewMode = 'detail';
      working = true;
      try {
        const fresh = await operate<Entity>({ operation: 'entity.get', input: { id } }, workspace.path);
        clearSelection(); selected = fresh;
        section = entitySection(fresh.kind,fresh.payload);
        loadDraft(fresh);
        await tick(); fitTitle();
        await loadRelations(fresh);
        await loadDashboard(fresh);
        await loadExternal(fresh);
      } catch (e) { error = report(e); }
      finally { working = false; }
  }
  async function newNote() {
    if (section === 'goals') await createHome('note',undefined,{inbox:false,icon:'trophy',goal:{state:'dream'}});
    else if (section === 'home') await createHome('note'); else await newEntity('note');
  }
  async function openHome(id: string) {
    if (onrequestdetail) {
      if (working || blocked) return;
      working = true;
      try { const entity = await operate<Entity>({ operation: 'entity.get', input: { id } }, workspace.path); await onrequestdetail({ type: 'entity', id, kind: entity.kind, title: entity.title }); }
      catch (e) { error = report(e); }
      finally { working = false; }
      return;
    }
    await guarded(async () => {
      if (!workspace) return;
      const path = workspace.path;
      working = true;
      viewMode = 'detail';
      try {
        const entity = await operate<Entity>({ operation: 'entity.get', input: { id } }, path);
        if (workspace?.path !== path) return;
        section = entitySection(entity.kind,entity.payload); await loadList(); await openOriginal(entity.id);
        await tick(); if (active && !blocked) editorRoot?.focus();
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function createHome(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
    if (onrequestdetail) { await onrequestdetail({ type: 'draft', kind, context, seed }); return; }
    await guarded(async () => {
      if (!workspace) return;
      working = true;
      viewMode = 'detail';
      try { section = kind; await loadList(); }
      finally { working = false; }
      await startDraft(kind, context, seed);
    });
  }
  async function newEntity(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
    if (!workspace) return;
    if (onrequestdetail) {
      const defaults = kind === 'tag' ? { classification: section === 'areas' ? 'area' : section === 'resources' ? 'resource' : 'topic' } : kind === 'task' && section === 'my_day' && workflowDate ? { planned_date: workflowDate } : {};
      await onrequestdetail({ type: 'draft', kind, context, seed: { ...defaults, ...seed } }); return;
    }
    await guarded(() => startDraft(kind, context, seed));
  }
  async function startDraft(kind: EntityKind, context?: ContextLink, seed?: Record<string, unknown>) {
      viewMode = 'detail';
      clearSelection(); draft = { kind, title: '', payload: kind === 'tag' ? { classification: section === 'areas' ? 'area' : section === 'resources' ? 'resource' : 'topic' } : kind === 'task' && section === 'my_day' && workflowDate ? { planned_date: workflowDate } : {}, context, ...(kind === 'note' ? { document: { schema_version: 2, blocks: [{ id: crypto.randomUUID(), type: 'paragraph', content: [] }] } } : {}) }; editorInitial = draft.document ? JSON.parse(JSON.stringify(draft.document)) as Document : kind === 'task' ? emptyEditorDocument() : null;
      if (seed) draft.payload = { ...draft.payload, ...seed };
      if (draft.kind === 'note' && draft.payload.goal && editorInitial) initializeGoalEditors(editorInitial);
      section = entitySection(kind,draft.payload);
      status = 'dirty'; await tick(); if (active && !blocked) titleInput?.focus(); fitTitle();
  }
  function edited() { if (!noteReadOnly) status = remoteRevision ? 'conflict' : 'dirty'; }
  async function save(): Promise<boolean> {
    if (working || status === 'saving') return false;
    if (collectionPending) return databasePanel ? await databasePanel.savePending() : false;
    if (!workspace || !draft || noteReadOnly) return false;
    if (invalidEditor) { error = "The editor returned an invalid format. Correct or copy the draft before leaving; no earlier version was saved."; status = 'error'; return false; }
    status = 'saving'; error = '';
    try {
      const document = draft.document ? JSON.parse(JSON.stringify(draft.document)) as Document : undefined;
      const payload = JSON.parse(JSON.stringify(draft.payload)) as Record<string, unknown>;
      const payloadChanged = JSON.stringify(payload) !== JSON.stringify(selected?.payload);
      const entity = selected
        ? await operate<Entity>({ operation: 'entity.update', input: { id: selected.id, expected_revision: selected.revision, title: draft.title, document, ...(payloadChanged ? { payload } : {}) } }, workspace.path)
        : await operate<Entity>({ operation: 'entity.create', input: { kind: draft.kind, title: draft.title, document, payload, context: draft.context } }, workspace.path);
      selected = entity; draft = { ...draft, title: entity.title, payload: JSON.parse(JSON.stringify(entity.payload)) as Record<string, unknown>, ...(entity.document ? { document: structuredClone(entity.document) as Document } : {}) };
      status = 'saved'; remoteRevision = null;
      await loadList();
      try { await loadRelations(entity); await loadDashboard(entity); await loadExternal(entity); } catch (e) { error = `Save confirmed; relations could not be refreshed: ${report(e)}`; }
      return true;
    } catch (e) {
      if (e instanceof OperationError && e.detail.code === 'revision_conflict') {
        status = 'conflict'; remoteRevision = Number((e.detail.details as { current_revision: number }).current_revision);
      } else status = 'error';
      error = report(e); return false;
    }
  }
  function overviewRefreshPanel() {
    if (section === 'home') return homePanel;
    if (viewMode !== 'overview') return undefined;
    return section === 'task' ? tasksPage : section === 'note' ? notesPage : section === 'project' ? projectsPage : section === 'tag' ? tagsPage : section === 'goals' ? goalsPage : section === 'person' ? peoplePage : undefined;
  }
  function refresh(): Promise<void> {
    if (disposed) return Promise.resolve();
    const panel = overviewRefreshPanel();
    // Home readers already coalesce by query; avoid nesting a second trailing
    // cycle around all sections when several focus/keyboard events arrive.
    if (panel && ready && !working && !blocked && status !== 'saving') return panel.refreshPanel();
    return refreshCoordinator.request();
  }
  async function performRefresh() {
    if (!workspace) return;
    const workspacePath = workspace.path;
    const requestedSection = section, requestedMode = viewMode, requestedScope = searchScope;
    const entityId = selected?.id;
    const current = () => !disposed && workspace?.path === workspacePath && section === requestedSection && viewMode === requestedMode && (requestedSection !== 'search' || searchScope === requestedScope);
    let panel = overviewRefreshPanel();
    if (!panel && (requestedSection === 'home' || (requestedMode === 'overview' && ['task', 'note', 'project', 'tag', 'goals', 'person'].includes(requestedSection)))) {
      await tick();
      if (!current()) return;
      panel = overviewRefreshPanel();
    }
    if (panel) { await panel.refreshPanel(); return; }
    if (requestedSection === 'search' && requestedScope === 'attachments') await attachmentPanel?.refreshPanel();
    else if (requestedSection === 'search' && requestedScope === 'combined') await groupedPanel?.refreshPanel();
    else await loadList();
    if (!current()) return;
    if (requestedSection === 'collections' && databasePanel) await databasePanel.refreshPanel();
    if (!current()) return;
    if (selected && goalMilestones) await goalMilestones.refreshPanel();
    if (!current() || !selected || selected.id !== entityId) return;
    detailRefreshController?.abort();
    const controller = detailRefreshController = new AbortController();
    try {
      const fresh = await operate<Entity>({ operation: 'entity.get', input: { id: selected.id } }, workspacePath, { signal: controller.signal });
      if (!current() || !selected || selected.id !== entityId || working || (status as Status) === 'saving' || fresh.revision < selected.revision) return;
      if (dirty) {
        if (fresh.revision !== selected.revision) { remoteRevision = fresh.revision; status = 'conflict'; error = "An external revision exists. Your pending content is still here."; }
      } else {
        if (fresh.revision !== selected.revision) { selected = fresh; loadDraft(fresh); }
      }
      await loadRelations(fresh);
      if (!current() || selected?.id !== entityId) return;
      await loadDashboard(fresh);
    } catch (e) { if (!controller.signal.aborted && current() && selected?.id === entityId) error = report(e); }
    finally { if (detailRefreshController === controller) detailRefreshController = undefined; }
  }
  async function reload() {
    const entity = selected;
    if (entity) await guarded(() => openOriginal(entity.id));
  }
  async function archive() {
    if (!selected || !workspace || noteReadOnly) return;
    await guarded(async () => {
      const entity = selected;
      if (!entity) return;
      working = true;
      try {
        await operate<Entity>({ operation: entity.archived_at ? 'entity.restore' : 'entity.archive', input: { id: entity.id, expected_revision: entity.revision } }, workspace!.path);
        clearSelection(); await loadList();
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function togglePageLock() {
    if (!plainNotePage || !selected || working || blocked || status === 'saving') return;
    const next = !pageLocked;
    await guarded(async () => {
      if (!selected) return;
      // Discard resolves the old draft first; restore its saved document here.
      if (!draft) loadDraft(selected);
      const entityId = selected.id;
      working = true; error = '';
      try {
        const entity = await operate<Entity>({ operation: 'entity.update', input: { id: entityId, expected_revision: selected.revision, payload: { page_locked: next } } }, workspace.path);
        if (disposed || selected?.id !== entityId) return;
        selected = entity;
        if (draft) draft.payload = structuredClone(entity.payload) as Record<string, unknown>;
        temporaryUnlockFor = null;
        status = 'saved'; remoteRevision = null;
      } catch (e) { error = report(e); }
      finally { working = false; }
    });
  }
  async function toggleTemporaryUnlock() {
    if (!selected || !pageLocked || working || blocked || status === 'saving') return;
    if (noteReadOnly) { temporaryUnlockFor = selected.id; return; }
    await guarded(async () => {
      if (!selected || (selected.payload as Record<string, unknown>).page_locked !== true) return;
      if (!draft) loadDraft(selected);
      temporaryUnlockFor = null;
      await tick(); lockStatusButton?.focus();
    });
  }
  async function copyPending() {
    recovery = invalidEditor || (draft?.document ? pendingJson(draft.title, draft.document) : JSON.stringify(draft ?? selected?.document, null, 2));
    try { await navigator.clipboard.writeText(recovery); } catch { error = "The content is in the recovery field: select and copy the text."; }
  }
  async function addRelation(target: Entity, kind: 'project' | 'tag' | 'person') {
    if (!workspace || !selected || working || status === 'saving' || noteReadOnly) return;
    const entityId = selected.id;
    working = true; relationError = '';
    try {
      const r = await operate<Contract['relation_result']>({ operation: 'relation.add', input: { source_id: entityId, target_id: target.id, kind, expected_revision: selected.revision } }, workspace.path);
      if (selected?.id !== entityId) return;
      selected = r.entity;
      try { await loadRelations(r.entity); await loadDashboard(r.entity); }
      catch (e) { relationError = `Relation saved, but the view could not be refreshed: ${report(e)}`; }
    } catch (e) { handleRelationError(e, true); }
    finally { working = false; }
  }
  async function removeRelation(relation: Relation): Promise<boolean> {
    if (!workspace || !selected || working || status === 'saving' || noteReadOnly) return false;
    const entityId = selected.id;
    working = true; relationError = '';
    try {
      const source = await operate<Entity>({ operation: 'entity.get', input: { id: relation.source_id } }, workspace.path);
      if (selected?.id !== entityId) return false;
      const r = await operate<Contract['relation_result']>({ operation: 'relation.remove', input: { id: relation.id, expected_revision: source.id === entityId ? selected.revision : source.revision } }, workspace.path);
      if (selected?.id !== entityId) return false;
      if (entityId === r.entity.id) selected = r.entity;
      try { await loadRelations(selected); await loadDashboard(selected); return true; }
      catch (e) { relationError = `Relation removed, but the view could not be refreshed: ${report(e)}`; return false; }
    } catch (e) { handleRelationError(e, relation.source_id === entityId); return false; }
    finally { working = false; }
  }
  function handleRelationError(e: unknown, sourceIsOpen: boolean) {
    relationError = report(e);
    if (sourceIsOpen && dirty && e instanceof OperationError && e.detail.code === 'revision_conflict') {
      const revision = Number((e.detail.details as { current_revision?: number } | null)?.current_revision);
      remoteRevision = Number.isFinite(revision) && revision > 0 ? revision : null;
      status = 'conflict'; error = 'An external revision exists. Your pending content is still here.';
    }
  }
  function downloadJson() {
    const url = URL.createObjectURL(new Blob([JSON.stringify(selected?.document, null, 2)], { type: 'application/json' }));
    const a = document.createElement('a'); a.href = url; a.download = `noteka-${selected?.id}.json`; a.click(); URL.revokeObjectURL(url);
  }
  async function exportNote(format: 'json' | 'markdown') {
    if (!workspace || !selected || dirty || !exportPath.trim()) return;
    try { const result = await operate<Contract['export']>({ operation: 'document.export', input: { id: selected.id, destination: exportPath.trim(), format } }, workspace.path); exportMessage = `Exported to ${result.path}. ${result.warnings.join(' ')}`; }
    catch (e) { error = report(e); }
  }
  function keydown(e: KeyboardEvent) {
    if (!active || blocked || e.defaultPrevented) return;
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') { e.preventDefault(); if (dirty) void save(); }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'n') { e.preventDefault(); void newNote(); }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'a') { e.preventDefault(); void archive(); }
    if (e.key === 'Escape' && guard) { e.preventDefault(); void resolveGuard('cancel'); }
  }
  function modal(node: HTMLDialogElement) {
    const previous = document.activeElement as HTMLElement;
    node.showModal(); node.querySelector<HTMLButtonElement>('[data-cancel]')?.focus();
    return { destroy() { previous?.focus(); } };
  }
  export async function openTarget(target: NavigationTarget): Promise<boolean> {
    if (target.type === 'section') return navigate(target.section);
    if (target.type === 'draft') return guarded(async () => { section = target.kind; await startDraft(target.kind, target.context, target.seed); });
    return guarded(async () => {
      working = true;
      try {
        viewMode = 'detail'; section = target.kind; clearSelection(); await loadList(); await openOriginal(target.id);
        if (selected?.id === target.id && target.hit) {
          const hit = target.hit;
          sourceLocator = `Source: ${hit.entity_id} · block ${hit.block_id ?? "title"} · result r${hit.revision}${selected.revision !== hit.revision ? ` · current source r${selected.revision}; search again` : ''}`;
          sourceBlock = hit.block_id && selected.revision === hit.revision ? hit.block_id : undefined;
        }
        await tick(); if (active && !blocked) editorRoot?.focus();
      } finally { working = false; }
    });
  }
  onMount(() => {
    void openTarget(initialTarget).finally(() => { ready = true; });
    return () => { disposed = true; refreshCoordinator.dispose(); listGeneration++; relationReadGeneration++; listController?.abort(); relationReadController?.abort(); dashboardReadController?.abort(); detailRefreshController?.abort(); guard?.resolve(false); };
  });
</script>

{#snippet relationSection()}
  {@const kind = selected?.kind}
  {@const kinds = (kind === 'note' ? (plainNotePage || goalDetail ? ['project', 'person'] : ['project', 'tag', 'person']) : kind === 'task' ? ['tag', 'person'] : kind === 'project' ? ['tag', 'person'] : kind === 'tag' ? ['person'] : kind === 'person' ? ['tag'] : []) as ('project' | 'tag' | 'person')[]}
  {@const shown = relationships.filter(item => !((plainNotePage || goalDetail) && item.relation.kind === 'tag' && item.other.kind === 'tag') && !(kind === 'task' && item.relation.kind === 'project' && item.other.kind === 'project'))}
  {@const other = shown.filter(item => !kinds.includes(item.other.kind as 'project' | 'tag' | 'person'))}
  <section class="relations page-relations" aria-label="Relations">
    <h3>Relations</h3>
    {#if relationError && !relationPicker}<p class="error" role="alert">{relationError}</p>{/if}
    {#each kinds as relationKind}
      {@const items = shown.filter(item => item.other.kind === relationKind)}
      <div class="relation-group">
        <span class="relation-group-label">{relationKind === 'project' ? 'Projects' : relationKind === 'tag' ? 'Tags' : 'People'}</span>
        <div class="relation-chips">
          {#each items as item (item.relation.id)}
            <span class="relation-chip-wrap"><button type="button" class="relation-chip" onclick={() => void openEntity(item.other)}><AppIcon name={entityIcon(item.other.kind, item.other.title, item.other.payload)} color={entityIconColor(item.other.payload)} size={14} />{item.other.title || 'Untitled'}{item.other.archived_at ? ' · archived' : ''}</button><button type="button" class="relation-remove" aria-label={`Remove relation to ${item.other.title}`} disabled={working || status === 'saving' || noteReadOnly} onclick={() => void removeRelation(item.relation)}><AppIcon name="close" size={12} /></button></span>
          {/each}
          <button type="button" class="relation-add" aria-haspopup="dialog" disabled={working || status === 'saving' || noteReadOnly} onclick={event => openPageRelations(relationKind, event.currentTarget)}><AppIcon name="plus" size={13} />Add {relationKind}</button>
        </div>
      </div>
    {/each}
    {#if other.length}<div class="relation-group"><span class="relation-group-label">Related pages</span><div class="relation-chips">{#each other as item (item.relation.id)}<span class="relation-chip-wrap"><button type="button" class="relation-chip" onclick={() => void openEntity(item.other)}><AppIcon name={entityIcon(item.other.kind,item.other.title,item.other.payload)} color={entityIconColor(item.other.payload)} size={14} />{item.other.title || 'Untitled'}{item.other.archived_at ? ' · archived' : ''}</button><button type="button" class="relation-remove" aria-label={`Remove relation to ${item.other.title}`} disabled={working || status === 'saving' || noteReadOnly} onclick={() => void removeRelation(item.relation)}><AppIcon name="close" size={12} /></button></span>{/each}</div></div>{/if}
  </section>
{/snippet}

        {#snippet pageToolContent(name: string)}
          {#if name === 'export'}
        {#if selected?.document && !dirty}<div class="export-tools"><label>New export directory<input class="glass-field" aria-label="Export destination" bind:value={exportPath} placeholder="/ruta/a/exportacion" /></label><button class="glass-secondary" disabled={!exportPath.trim()} onclick={() => void exportNote('json')}>Export JSON and originals</button><button class="glass-secondary" disabled={!selected.document_editable || !exportPath.trim()} onclick={() => void exportNote('markdown')}>Export Markdown</button>{#if exportMessage}<p role="status">{exportMessage}</p>{/if}</div>{/if}
          {:else if name === 'resources'}
        {#if selected}
          <div class="external-resources" aria-label="External resources">
            <div class="actions"><button class="glass-secondary" disabled={dirty || working || externalBusy || noteReadOnly} onclick={() => void linkGoogleDrive()}>{externalBusy ? 'Esperando Google Picker…' : "Link from Google Drive"}</button></div>
            {#if externalError}<p class="error" role="alert">{externalError}</p>{/if}
            {#if externalResources.length}
              {#each externalResources as resource (resource.id)}
                <div class="external-row">
                  <div><strong>{externalTitle(resource)}</strong><small class="metadata">{resource.provider} · {resource.resource_type} · {resource.resource_id}</small></div>
                  <div class="actions">
                    {#if externalCanPreview(resource)}<button class="glass-secondary" disabled={externalBusy || externalPreviewBusy} onclick={() => void previewResource(resource)}>Preview</button>{/if}
                    {#if externalLink(resource)}<button class="glass-secondary" disabled={externalBusy || externalPreviewBusy} onclick={() => void openResource(resource)}>Open</button>{/if}
                    <button class="glass-secondary" disabled={dirty || working || externalBusy || externalPreviewBusy || noteReadOnly} onclick={() => void unlinkExternal(resource)}>Unlink</button>
                  </div>
                </div>
              {/each}
            {:else}<p class="metadata">No external resources linked.</p>{/if}
            {#if externalPreviewError}<p class="error" role="alert">{externalPreviewError}</p>{/if}
            {#if externalPreview}
              <section class="external-preview" aria-label="External preview">
                <div class="external-row"><strong>{externalPreview.name || externalPreview.id}</strong><button class="glass-secondary" onclick={closePreview}>Close</button></div>
                <pre class="external-preview-text">{externalPreview.text}</pre>
                <p class="metadata">{externalPreview.preview_mime === 'text/csv' ? "First sheet exported as CSV; formulas are not recalculated." : "Document exported as plain text."} Read-only preview; not saved to the workspace.</p>
              </section>
            {/if}
            <p class="metadata">Google Picker usa `drive.file`; no descarga ni edita archivos.</p>
          </div>
        {/if}

          {:else if name === 'attachments' && pageKind === 'note'}<div class="local-tools">{#key `${workspace.path}:${selected?.id ?? 'draft'}:${editorEpoch}`}<AttachmentPreparationPanel workspace={workspace.path} document={draft?.document} /><AttachmentGenerationPanel workspace={workspace.path} />{/key}</div>{/if}
        {/snippet}

<svelte:window onkeydown={keydown} onfocus={() => { if (active || visible) void refresh(); }} onresize={() => { if (active) void tick().then(fitTitle); }} />

<div class="workspace-pane" class:home-pane={section === 'home'} class:overview-pane={overview} class:breadcrumb-overview={section === 'home' || overview} aria-label={currentLabel}>
    {#if section === 'home' || overview}<div class="overview-breadcrumb-bar"><PageBreadcrumb {section} disabled={blocked || working || status === 'saving'} onnavigate={navigate} /></div>{/if}
    {#if section === 'home'}
      {#key workspace.path}<HomePanel bind:this={homePanel} workspace={workspace.path} {active} {visible} {animateAlongsideSidePeek} {blocked} {searchOpen} {error} onopen={openHome} oncreate={createHome} onnavigate={async (kind) => { await navigate(kind); }} />{/key}
    {:else if section === 'task' && overview}
      {#key workspace.path}<TasksPage bind:this={tasksPage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} onhome={async () => { await navigate('home'); }} onprojects={async () => { await navigate('project'); }} />{/key}
    {:else if section === 'note' && overview}
      {#key workspace.path}<NotesPage bind:this={notesPage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} ontags={async () => { await navigate('tag'); }} />{/key}
    {:else if section === 'project' && overview}
      {#key workspace.path}<ProjectsPage bind:this={projectsPage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} />{/key}
    {:else if section === 'tag' && overview}
      {#key workspace.path}<TagsPage bind:this={tagsPage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} onsection={async (kind) => { await navigate(kind); }} />{/key}
    {:else if section === 'goals' && overview}
      {#key workspace.path}<GoalsPage bind:this={goalsPage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} onsection={async (kind) => { await navigate(kind); }} />{/key}
    {:else if section === 'person' && overview}
      {#key workspace.path}<PeoplePage bind:this={peoplePage} workspace={workspace.path} {active} {blocked} {error} onopen={openHome} oncreate={createHome} />{/key}
    {:else}
    {#if overview}<section class="entity-list" id={`entity-list-${paneId}`} aria-label={currentLabel}>
      <header><h2><span class="section-icon" aria-hidden="true"><AppIcon name={sectionIcon()} size={18} /></span>{currentLabel}</h2><button title="Refresh" aria-label="Refresh" onclick={() => void refresh()} disabled={loading || working}><AppIcon name="refresh" size={16} /></button></header>
      {#if section !== 'archive' && section !== 'search' && section !== 'collections'}<button class="new-note" onclick={() => void newEntity(createKind)}><AppIcon name="plus" size={16} />{createLabel[createKind]} {#if createKind === 'note'}<kbd>⌘N</kbd>{/if}</button>{/if}
      {#if section === 'search'}
        <label>Search scope<select class="glass-field" aria-label="Search scope" value={searchScope} disabled={working || status === 'saving'} onchange={(e) => changeSearchScope(e.currentTarget.value as 'notes' | 'attachments' | 'combined')}><option value="notes">Notes and entities</option><option value="attachments">Attachments</option><option value="combined">Notes and attachments (groups)</option></select></label>
        {#if searchScope === 'notes'}<form class="search-form" onsubmit={(e) => { e.preventDefault(); void loadList(); }}><label>Local text<input class="glass-field" aria-label="Search text" bind:value={searchQuery} oninput={invalidateSearchCriteria} placeholder="1–16 terms" maxlength="256" /></label><label>Type<select class="glass-field" aria-label="Search type" bind:value={searchKind} onchange={invalidateSearchCriteria}><option value="">All</option>{#each ['note','task','project','tag','person'] as kind}<option value={kind}>{kind}</option>{/each}</select></label><label class="checkbox-label"><input type="checkbox" bind:checked={searchArchived} onchange={invalidateSearchCriteria} /> Archived only</label><label>ID contexto opcional<input class="glass-field" aria-label="Search context" bind:value={searchRelated} oninput={invalidateSearchCriteria} /></label><button class="glass-secondary" disabled={loading || working || !searchQuery.trim()}>Search</button><button class="glass-secondary" type="button" disabled={working || loading} onclick={() => void rebuildSearch()}>Rebuild index</button><p class="list-message">Titles and recognized blocks. Choose Attachments for selected file fragments. No models.</p>{#if searchMessage}<p class="list-message" role="status">{searchMessage}</p>{/if}</form>
        {:else if searchScope === 'attachments'}{#key workspace.path}<AttachmentSearchPanel bind:this={attachmentPanel} workspace={workspace.path} />{/key}{:else}{#key workspace.path}<GroupedSearchPanel bind:this={groupedPanel} workspace={workspace.path} onopen={openSearch} />{/key}{/if}
      {/if}
      {#if workflowDate}<p class="list-message">Calendar day {workflowDate} · {workspace.timezone}</p>{/if}
      {#if loading && entities.length === 0}<p class="list-message">Loading…</p>{/if}
      {#if listError}<p class="error" role="alert">{listError}</p><button onclick={() => void loadList()}>Retry</button>{/if}
      {#if listStale}<p class="list-message" role="status">{loading ? "Updating; showing previous results." : "Showing previous results; they may be out of date."}</p>{/if}
      {#if listNotice && !loading}<p class="list-message" role="status">{listNotice}</p>{/if}
      {#if !loading && !listError && entities.length === 0 && section !== 'search'}<p class="list-message">{section === 'collections' ? "Collections and views in the main panel. Each record opens its original entity." : section === 'archive' ? "Nothing archived." : "No items yet."}</p>{/if}
      <div class="list-items">
        {#if section === 'search' && searchScope === 'notes'}{#each searchHits as hit, i (`${hit.entity_id}:${hit.block_id ?? ''}:${i}`)}<button class="search-hit" onclick={() => void openSearch(hit)}><span class="entity-title">{hit.title}</span><small>{hit.block_id ? `Block ${hit.block_id}` : "Title"} · r{hit.revision}</small><span class="search-excerpt">{hit.excerpt}</span></button>{/each}{#if !loading && searchQuery.trim() && !searchHits.length && !listError}<p class="list-message">No results.</p>{/if}{/if}
        {#each entities as entity (entity.id)}
          <button class:selected={selected?.id === entity.id} onclick={() => void openEntity(entity)} disabled={working || status === 'saving'}><span class="entity-icon" aria-hidden="true"><AppIcon name={entityIcon(entity.kind, entity.title, entity.payload as Record<string, unknown>)} color={entityIconColor(entity.payload as Record<string, unknown>)} size={16} /></span><span class="entity-title">{entity.title}</span><small>{entity.kind} · r{entity.revision}</small></button>
        {/each}
      </div>
      {#if nextCursor}<button onclick={() => void loadList(nextCursor!)} disabled={loading || listStale}>Load more</button>{/if}
    </section>{/if}
    <main class="editor" class:has-scroll-outline={outlineNote} bind:this={editorRoot} tabindex="-1" aria-label={`Details for ${selected?.title || draft?.title || currentLabel}`} hidden={overview}>
      <header class="editor-bar">{#if onclosepeek}<button class="icon-button close-peek" aria-label="Close peek" title="Close peek (Esc)" disabled={blocked || working || status === 'saving'} onclick={onclosepeek}><AppIcon name="forward" size={18} /></button>{/if}
        {#if onpresent && ondefaultpeek && (draft || selected)}<PeekModeMenu mode={peekMode} defaultMode={defaultPeekMode} preferenceError={peekPreferenceError} disabled={blocked || working || status === 'saving'} onchange={onpresent} ondefault={ondefaultpeek} />{/if}
        <PageBreadcrumb {section} record={breadcrumbRecord} disabled={blocked || working || status === 'saving'} onnavigate={navigate} />
        <div class="actions note-page-actions" role="group" aria-label={plainNotePage ? 'Note actions' : 'Page actions'}>
          {#if plainNotePage}
            {#if pageLocked}<button bind:this={lockStatusButton} type="button" class="page-lock-status" class:temporarily-unlocked={!noteReadOnly} aria-label={noteReadOnly ? "Unlock page temporarily" : "Re-lock page"} title={noteReadOnly ? "Unlock temporarily to edit" : "Lock the page again"} disabled={blocked || working || status === 'saving'} onclick={() => void toggleTemporaryUnlock()}><AppIcon name={noteReadOnly ? "lock" : "lock-open"} size={13} /><span>{noteReadOnly ? "Locked" : "Re-lock"}</span></button>{/if}
          {/if}
          <button class="page-refresh" aria-label="Refresh page" title="Refresh page" disabled={working || loading || status === 'saving'} onclick={() => void refresh()}><AppIcon name="refresh" size={16} /></button>
          {#if draft}<span class:warning={dirty} class="save-status" role="status">{({ saved: "Saved", dirty: "Unsaved changes", saving: "Saving…", error: "Error · Unsaved", conflict: "Conflict · Unsaved" })[status]}</span>{#if dirty}<button class="primary glass-action" onclick={() => void save()} disabled={working || status === 'saving'}>{status === 'error' ? "Retry" : "Save"} <kbd>⌘S</kbd></button>{/if}{/if}
          {#if selected || draft}<PageTools groups={toolGroups} bind:active={utilityPanel} content={pageToolContent} placement="header" actions={[
            ...(plainNotePage ? [{ id: 'lock', label: 'Lock page', icon: 'lock', checked: pageLocked, disabled: !selected || working || status === 'saving', reason: !selected ? 'Save the page to lock it' : undefined, onselect: () => { void togglePageLock(); } }] : []),
            ...(selected ? [{ id: 'archive', label: selected.archived_at ? 'Restore' : 'Archive', icon: selected.archived_at ? 'undo' : 'archive', disabled: working || status === 'saving' || noteReadOnly, reason: noteReadOnly ? 'Unlock the page first' : undefined, onselect: () => { void archive(); } }] : [])
          ]} />{/if}
        </div>
      </header>
      {#if projectDetail || tagDetail}<div class="project-page-cover" aria-hidden="true"></div>{/if}
      {#if personDetail}<div class="person-detail-cover" aria-hidden="true"><PersonPhoto id={typeof personPayload.photo === 'string' ? personPayload.photo : null} position={typeof personPayload.photo_position === 'number' ? personPayload.photo_position : 50} /></div>{/if}
      <div class="page" class:goal-detail={goalDetail} class:person-detail={personDetail} class:project-detail={projectDetail} class:tag-detail={tagDetail} class:task-detail={(draft?.kind ?? selected?.kind) === 'task'} class:note-detail={plainNotePage}>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        
        {#if status === 'conflict'}<div class="conflict"><p>Local revision: {selected?.revision}. Current revision: {remoteRevision}. No automatic merge.</p><div class="actions"><button onclick={() => void reload()}>Reload saved version</button><button onclick={() => void copyPending()}>Copy pending content</button></div></div>{/if}
        {#if section === 'collections' && !selected && !draft}
          <DatabasePanel bind:this={databasePanel} workspace={workspace.path} onpending={(pending) => collectionPending = pending} onbusy={(busy) => working = busy} onguard={async action => { await guarded(action); }} onopen={openEntity} onerror={(message) => error = message} />
        {:else if draft}
          <div class="page-title-row">{#if draft.kind === 'task' && !draft.payload.icon}<div class="task-title-tools"><IconPicker size={18} icon="task" color={entityIconColor(draft.payload) ?? null} onchange={chooseIcon} /><span>Add icon</span></div>{:else}<IconPicker disabled={plainNotePage && noteEditingDisabled} size={draft.kind === 'project' ? 88 : draft.kind === 'tag' || draft.kind === 'person' ? 72 : 62} icon={entityIcon(draft.kind, draft.title, draft.payload)} color={entityIconColor(draft.payload) ?? null} onchange={chooseIcon} />{/if}<textarea class="title-input" readonly={noteReadOnly} rows="1" aria-label={draft.kind === 'note' ? draft.payload.goal ? "Goal title" : draft.payload.milestone ? "Milestone title" : "Note title" : "Entity title"} placeholder="Untitled" bind:this={titleInput} bind:value={draft.title} oninput={(e) => { edited(); fitTitle(); }} onkeydown={titleKeydown} disabled={status === 'saving' || working}></textarea></div>
          <details class="page-details" bind:open={detailsOpen}><summary>View details</summary>
            {#if goalDetail}<EntityFields workspace={workspace.path} kind="note" payload={draft.payload} goalSecondaryOnly disabled={noteEditingDisabled} onchange={edited} />{:else if personDetail}<EntityFields workspace={workspace.path} kind="person" payload={draft.payload} disabled={noteEditingDisabled} onchange={edited} />{:else if draft.kind === 'task'}<EntityFields workspace={workspace.path} kind="task" payload={draft.payload} taskSecondaryOnly disabled={noteEditingDisabled} onchange={edited} />{:else if plainNotePage}<EntityFields workspace={workspace.path} kind="note" payload={draft.payload} noteSecondaryOnly disabled={noteEditingDisabled} onchange={edited} />{:else if draft.kind === 'project'}<EntityFields workspace={workspace.path} kind="project" payload={draft.payload} projectSecondaryOnly disabled={noteEditingDisabled} onchange={edited} />{:else if draft.kind === 'tag'}<EntityFields workspace={workspace.path} kind="tag" payload={draft.payload} tagSecondaryOnly disabled={noteEditingDisabled} onchange={edited} />{/if}
            {#if selected}<p class="metadata">Revision {selected.revision} · {new Date(selected.updated_at).toLocaleString('en-US', { timeZone: workspace.timezone })} · {workspace.timezone} · Local{selected.archived_at ? " · Archived" : ''}</p>{/if}
            {#if sourceLocator}<p class="source-locator" role="status">{sourceLocator}</p>{/if}
            {#if importInfo}<dl class="imported-details">{#each importInfo.properties as property}<dt>{property.name}</dt><dd>{property.value}</dd>{/each}</dl><p class="metadata">Preserved properties from the imported document. Read only; editing content preserves its original blocks.</p>{/if}
          </details>
          {#if primaryImported.length}<dl class="page-properties" aria-label="Imported document properties">{#each primaryImported as property}<div><dt>{#if property.name === 'Favorite'}<AppIcon name="star" size={13} />{:else if ['Tag','Tags'].includes(property.name)}<AppIcon name="tag" size={13} />{:else}<AppIcon name="circle" size={12} />{/if}{property.name}</dt><dd>{#if propertyStyle(property.name) === 'checkbox'}<input type="checkbox" disabled checked={['yes','true',"yes"].includes(property.value.toLowerCase())} aria-label={property.name} />{:else if propertyStyle(property.name) === 'chip'}<span class="property-chip">{property.value}</span>{:else}{property.value}{/if}</dd></div>{/each}</dl>{/if}
          {#if draft.context}<p class="metadata">Captura vinculada a {draft.context.kind}: {draft.context.target_id}</p>{/if}
          {#if goalDetail}<GoalPageProperties payload={draft.payload} tags={noteTags} summary={goalSummary} busy={working || goalSummaryBusy} error={goalSummaryError} disabled={noteEditingDisabled} canRelate={selected !== null} onchange={edited} onopen={openEntity} oneditrelations={(anchor) => openPageRelations('tag', anchor)} onretry={() => { if (selected) void loadGoalSummary(selected); }} />{:else if draft.kind === 'task'}<TaskPageProperties payload={draft.payload} projects={taskProjects} disabled={noteEditingDisabled} canRelate={selected !== null} captureProject={draft.context?.kind === 'project'} onchange={edited} onopen={openEntity} oneditrelations={(anchor) => openPageRelations('project', anchor)} />{:else if plainNotePage}<NotePageProperties readOnly={noteReadOnly} payload={draft.payload} tags={noteTags} sourceUrl={noteSourceUrl} disabled={status === 'saving' || working} canRelate={selected !== null} captureTag={draft.context?.kind === 'tag'} onchange={edited} onopen={openEntity} oneditrelations={(anchor) => openPageRelations('tag', anchor)} />{:else if draft.kind === 'project'}<ProjectPageProperties payload={draft.payload} summary={projectSummary} busy={working || projectSummaryBusy} error={projectSummaryError} disabled={noteEditingDisabled} onchange={edited} onretry={() => { if (selected) void loadProjectSummary(selected); }} />{:else if draft.kind === 'tag'}<TagPageProperties payload={draft.payload} summary={tagSummary} busy={working || tagSummaryBusy} error={tagSummaryError} disabled={noteEditingDisabled} onchange={edited} onretry={() => { if (selected) void loadTagSummary(selected); }} onnotes={focusTagNotes} />{:else if !personDetail && (!(draft.kind === 'note' && importInfo) || typedGoalPage)}<EntityFields workspace={workspace.path} kind={draft.kind} payload={draft.payload} disabled={noteEditingDisabled} onchange={edited} />{/if}
        {#if selected}{@render relationSection()}{/if}
          {#if goalDetail}<nav class="goal-detail-nav" aria-label="Goal navigation"><span>Nav</span><span aria-hidden="true">|</span><button onclick={() => void navigate('goals')}>Goals</button><span aria-hidden="true">|</span><button onclick={() => void navigate('home')}>Dashboard</button></nav>{/if}
          {#if goalDetail}
            <section class="goal-overview" aria-label="Goal Overview">
              <div class="goal-overview-heading"><AppIcon name="trophy" size={19} /><h2>Goal Overview</h2></div>
              {#key editorEpoch}{#if goalOverviewInitial}<BlockEditor label="Goal overview" placeholder="Describe what you want to achieve…" initial={goalOverviewInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={noteEditingDisabled} onopen={openEntity} onchange={document => goalChanged('overview', document)} onerror={message => error = message} oninvalid={(raw, message) => goalInvalid('overview', raw, message)} />{/if}{/key}
              <details class="goal-additional-details" bind:open={goalDetailsOpen}>
                <summary>Additional Details</summary>
                {#key editorEpoch}{#if goalDetailsInitial}<BlockEditor label="Additional goal details" placeholder="Add context and details…" initial={goalDetailsInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={noteEditingDisabled} onopen={openEntity} onchange={document => goalChanged('details', document)} onerror={message => error = message} oninvalid={(raw, message) => goalInvalid('details', raw, message)} />{/if}{/key}
              </details>
            </section>
            {#if selected}{#key selected.id}<MilestonesPanel bind:this={goalMilestones} workspace={workspace.path} goalId={selected.id} revision={selected.revision} onopen={openHome} oncreate={(goalId) => createHome('note',undefined,{inbox:false,icon:'target',milestone:{goal_id:goalId}})} />{/key}{/if}
            <details class="goal-journal" aria-label="Goal Journal" open>
              <summary><h2>Journal</h2></summary>
              <header><button type="button" class="goal-journal-add glass-secondary" disabled={noteEditingDisabled} onclick={addGoalLog}><AppIcon name="plus" size={16} />New entry</button></header>
              {#key editorEpoch}{#if goalJournalInitial}<BlockEditor bind:this={goalJournalEditor} label="Goal journal" placeholder="Record your progress and reflections…" initial={goalJournalInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={noteEditingDisabled} onopen={openEntity} onchange={document => goalChanged('journal', document)} onerror={message => error = message} oninvalid={(raw, message) => goalInvalid('journal', raw, message)} />{/if}{/key}
            </details>
            {#if selected}<GoalProjects projects={goalProjects} timezone={workspace.timezone} disabled={noteEditingDisabled} onopen={openEntity} onadd={anchor => openPageRelations('project', anchor)} />{/if}
            {#if goalLegacyVisible && goalLegacyInitial}
              <details class="goal-previous-content" open><summary>Previous content</summary>
                {#key editorEpoch}<BlockEditor label="Previous goal content" initial={goalLegacyInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={noteEditingDisabled} onopen={openEntity} onchange={document => goalChanged('legacy', document)} onerror={message => error = message} oninvalid={(raw, message) => goalInvalid('legacy', raw, message)} />{/key}
              </details>
            {/if}
          {:else}
            <section class:task-document={draft.kind === 'task'} class:note-document={plainNotePage} aria-label={draft.kind === 'task' ? "Task content" : "Document"}>{#key editorEpoch}{#if editorInitial}<BlockEditor bind:this={bodyEditor} label={draft.kind === 'task' ? "Task content" : "Note content"} placeholder={draft.kind === 'task' || plainNotePage ? "Write here. Use / to add blocks." : ''} showStarters={draft.kind === 'task' || plainNotePage} starterContext={plainNotePage ? 'note' : 'task'} pageStyle={draft.kind === 'task' || plainNotePage} initial={draft.kind === 'task' ? (draft.document ?? editorInitial) : editorInitial} workspace={workspace.path} focusBlock={sourceBlock} disabled={noteEditingDisabled} onopen={openEntity} onchange={(document) => { if (draft && !noteReadOnly) { invalidEditor = ''; draft.document = composeImportedBody(importInfo?.prefix ?? [], document); edited(); } }} onerror={(message) => { error = message; }} oninvalid={(raw, message) => { if (noteReadOnly) return; invalidEditor = importInfo ? JSON.stringify({ imported_prefix: importInfo.prefix, editor_buffer: JSON.parse(raw) }, null, 2) : raw; error = message; edited(); }} />{/if}{/key}</section>
          {/if}
        {:else if selected}
          <p class="eyebrow">{selected.kind}{selected.archived_at ? " · ARCHIVED" : ''}</p><div class="page-title-row"><span class="page-icon" aria-hidden="true"><AppIcon name={entityIcon(selected.kind, selected.title, selected.payload as Record<string, unknown>)} color={entityIconColor(selected.payload as Record<string, unknown>)} size={62} /></span><h1>{selected.title}</h1></div><p class="metadata">Revision {selected.revision} · Local</p>
          {#if (selected.kind === 'note' && !selected.document_editable) || (selected.kind === 'task' && selected.document && !selected.document_editable)}
            <div class="conflict"><h3>Read-only format</h3><p>This version does not recognize the document. It will not convert it or discard fields.</p><button onclick={() => { recovery = JSON.stringify(selected?.document, null, 2); }}>Ver JSON original</button> <button onclick={downloadJson}>Descargar JSON</button></div>
          {:else}<pre class="properties">{JSON.stringify(selected.payload, null, 2)}</pre>{/if}
        {:else if working}<p class="editor-loading" role="status">Opening page…</p>
        {:else}
          <div class="editor-empty"><span class="empty-icon" aria-hidden="true"><AppIcon name="file-text" size={46} /></span><h1>Space to think.</h1><p>Choose a section in the sidebar or search for an item.</p>{#if section === 'note'}<button onclick={() => void newNote()}>Create a note</button>{/if}</div>
        {/if}
        {#if selected && !draft}{@render relationSection()}{/if}
        {#if selected?.kind === 'project'}<nav class="project-detail-nav" aria-label="Project navigation"><span>Nav</span><span aria-hidden="true">|</span><button onclick={() => void navigate('project')}>Projects</button><span aria-hidden="true">|</span><button onclick={() => void navigate('home')}>Dashboard</button></nav><ProjectContents items={dashboard} workspace={workspace.path} timezone={workspace.timezone} hasMore={!!dashboardCursor} busy={dashboardBusy || working} stale={dashboardStale} notice={dashboardNotice} error={dashboardError} createDisabled={dirty || working} onopen={openEntity} oncreate={(kind, seed) => { if (selected) void newEntity(kind, { target_id: selected.id, kind: 'project' }, seed); }} onchanged={async () => { if (selected?.kind === 'project') await loadDashboard(selected); }} onmore={() => { if (selected && dashboardCursor) void loadDashboard(selected, dashboardCursor); }} onretry={() => { if (selected) void loadDashboard(selected); }} />{/if}
        {#if selected?.kind === 'tag'}<nav class="tag-detail-nav" aria-label="Tag navigation"><span>Nav</span><span aria-hidden="true">|</span><button onclick={() => void navigate('note')}>Notes</button><span aria-hidden="true">|</span><button onclick={() => void navigate('tag')}>Tags</button></nav><TagContents items={dashboard} hasMore={!!dashboardCursor} busy={dashboardBusy || working} stale={dashboardStale} notice={dashboardNotice} error={dashboardError} createDisabled={dirty || working} onopen={openEntity} oncreate={kind => { if (selected) void createHome(kind === 'task' ? 'task' : 'note', { target_id: selected.id, kind: 'tag' }, kind === 'clip' ? { note_type: 'web_clip' } : {}); }} onmore={() => { if (selected && dashboardCursor) void loadDashboard(selected, dashboardCursor); }} onretry={() => { if (selected) void loadDashboard(selected); }} />{/if}
        {#if selected?.kind === 'person'}<nav class="person-detail-nav" aria-label="Person navigation"><span>Nav</span><span aria-hidden="true">|</span><button onclick={() => void navigate('person')}>People</button><span aria-hidden="true">|</span><button onclick={() => void navigate('note')}>Notes</button></nav><PersonContents items={dashboard} hasMore={!!dashboardCursor} busy={dashboardBusy || working} stale={dashboardStale} notice={dashboardNotice} error={dashboardError} createDisabled={dirty || working} onopen={openEntity} oncreate={kind => { if (selected) void newEntity(kind, { target_id: selected.id, kind: 'person' }); }} onmore={() => { if (selected && dashboardCursor) void loadDashboard(selected, dashboardCursor); }} onretry={() => { if (selected) void loadDashboard(selected); }} />{/if}

        {#if recovery}<label for={`recovery-${paneId}`}>Recoverable content</label><textarea id={`recovery-${paneId}`} class="recovery" readonly value={recovery}></textarea>{/if}
      </div>
    </main>
    {#if outlineNote}<ScrollOutline root={editorRoot} active={visible && !overview} disabled={blocked || working || status === 'saving'} />{/if}
    {/if}
</div>

{#if relationPicker && selected && !noteReadOnly}
  <RelationPicker workspace={workspace.path} sourceId={selected.id} kind={relationPicker.kind} anchor={relationPicker.anchor} {relationships} busy={working || status === 'saving'} error={relationError} onadd={addRelation} onremove={removeRelation} onclose={() => { relationPicker = null; }} />
{/if}
{#if guard}
    <dialog use:modal aria-labelledby={`pending-title-${paneId}`} class="modal unsaved-changes-dialog glass-popup" oncancel={(e) => { e.preventDefault(); void resolveGuard('cancel'); }}>
      <h2 id={`pending-title-${paneId}`}>You have unsaved changes</h2><p>Choose what to do before leaving the current draft.</p>
      {#if error}<p class="error" role="alert">{error}</p>{/if}
      <div class="actions unsaved-actions">
        <button class="glass-secondary" data-cancel disabled={working || status === 'saving'} onclick={() => void resolveGuard('cancel')}><AppIcon name="close" size={16} />Cancel</button>
        <button class="glass-secondary" onclick={() => void resolveGuard('discard')} disabled={working || status === 'saving'}><AppIcon name="trash" size={16} />Discard</button>
        <button class="primary glass-action" onclick={() => void resolveGuard('save')} disabled={working || status === 'saving'}><AppIcon name="save" size={16} />Save and continue</button>
      </div>
    </dialog>
{/if}
