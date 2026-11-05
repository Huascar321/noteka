import type { WorkspaceInfo } from './contracts.generated';

// Preferencia local del workspace: la ruta no vive en SQLite para que el
// arranque pueda abrirla antes de tener una base.
export const WORKSPACE_PATH_KEY = 'noteka.workspace-path.v1';
type Store = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;
export type WorkspaceOpener = (path: string | undefined) => Promise<WorkspaceInfo>;
export type StartupResult = { workspace: WorkspaceInfo | null; error: string };

export function readWorkspacePath(store: () => Store): string {
  try { return store().getItem(WORKSPACE_PATH_KEY)?.trim() ?? ''; } catch { return ''; }
}
export function saveWorkspacePath(path: string, store: () => Store): string {
  try { store().setItem(WORKSPACE_PATH_KEY, path); return ''; } catch { return 'Workspace opened. Its path could not be saved on this device.'; }
}
export function clearWorkspacePath(store: () => Store): void {
  try { store().removeItem(WORKSPACE_PATH_KEY); } catch { /* la próxima lectura vuelve a intentar */ }
}
function errorCode(error: unknown): string {
  const detail = (error as { detail?: { code?: unknown } } | null)?.detail;
  return typeof detail?.code === 'string' ? detail.code : '';
}
function describe(error: unknown): string {
  return error instanceof Error && error.message ? error.message : 'The workspace could not be opened.';
}

// Último workspace guardado y, si esa ruta ya no contiene un workspace, el
// directorio de datos predeterminado (ruta omitida). Un error distinto de
// `not_found` no se sustituye por otro workspace: se informa.
export async function openStartupWorkspace(lastPath: string, open: WorkspaceOpener): Promise<StartupResult> {
  let fallbackReason: unknown = null;
  if (lastPath) {
    try { return { workspace: await open(lastPath), error: '' }; }
    catch (error) {
      if (errorCode(error) !== 'not_found') return { workspace: null, error: describe(error) };
      fallbackReason = error;
    }
  }
  try { return { workspace: await open(undefined), error: '' }; }
  catch (error) {
    if (errorCode(error) === 'not_found') return { workspace: null, error: '' };
    return { workspace: null, error: describe(error ?? fallbackReason) };
  }
}
