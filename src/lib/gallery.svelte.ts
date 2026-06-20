import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type {
  DuplicateGroup,
  GroupKey,
  HistoryEntry,
  KindFilter,
  MediaEntry,
  SortDir,
  SortKey,
  TrashEntry,
} from "./types";

const nameCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
const SKIP_DELETE_CONFIRM_KEY = "viewgallery:skipDeleteConfirm";
const FAVORITES_KEY = "viewgallery:favorites";
const MIN_ZOOM = 0.25;
const MAX_ZOOM = 5;
const ZOOM_STEP = 0.25;
const HISTORY_DEBOUNCE_MS = 400;
const MAX_UNDO_BATCHES = 10;

function loadFavorites(): Set<string> {
  if (typeof localStorage === "undefined") return new Set();
  try {
    const raw = localStorage.getItem(FAVORITES_KEY);
    return raw ? new Set(JSON.parse(raw)) : new Set();
  } catch {
    return new Set();
  }
}

class GalleryStore {
  folder = $state<string | null>(null);
  allItems = $state<MediaEntry[]>([]);
  searchText = $state("");
  kindFilter = $state<KindFilter>("all");
  sortKey = $state<SortKey>("name");
  sortDir = $state<SortDir>("asc");
  currentPath = $state<string | null>(null);
  view = $state<"viewer" | "grid">("viewer");
  loading = $state(false);
  error = $state<string | null>(null);

  zoom = $state(1);
  selectedPaths = $state<Set<string>>(new Set());
  selectionAnchor = $state<string | null>(null);
  groupBy = $state<GroupKey>("none");
  showHelp = $state(false);
  pendingDelete = $state<string[] | null>(null);
  skipDeleteConfirm = $state(
    typeof localStorage !== "undefined" && localStorage.getItem(SKIP_DELETE_CONFIRM_KEY) === "true",
  );
  duplicateGroups = $state<DuplicateGroup[]>([]);
  showDuplicates = $state(false);
  findingDuplicates = $state(false);
  siblingFolders = $state<string[]>([]);
  favorites = $state<Set<string>>(loadFavorites());
  favoritesOnly = $state(false);

  history = $state<HistoryEntry[]>([]);
  undoStack = $state<TrashEntry[][]>([]);
  pendingClearCache = $state(false);
  private historyTimer: ReturnType<typeof setTimeout> | undefined;

  filteredItems = $derived.by(() => {
    const q = this.searchText.trim().toLowerCase();
    const filtered = this.allItems.filter((item) => {
      if (this.kindFilter !== "all" && item.kind !== this.kindFilter) return false;
      if (this.favoritesOnly && !this.favorites.has(item.path)) return false;
      if (q && !item.name.toLowerCase().includes(q)) return false;
      return true;
    });

    const dir = this.sortDir === "asc" ? 1 : -1;
    const key = this.sortKey;
    return filtered.slice().sort((a, b) => {
      let cmp = 0;
      if (key === "name") cmp = nameCollator.compare(a.name, b.name);
      else if (key === "date") cmp = a.modified - b.modified;
      else cmp = a.size - b.size;
      return cmp * dir;
    });
  });

  currentIndex = $derived.by(() => {
    if (!this.currentPath) return -1;
    return this.filteredItems.findIndex((i) => i.path === this.currentPath);
  });

  current = $derived.by(() => {
    const idx = this.currentIndex;
    return idx >= 0 ? this.filteredItems[idx] : null;
  });

  /** Opens a folder, restoring the last-viewed item from history when no explicit focus is given. */
  async openFolder(folderPath: string, focusPath?: string) {
    this.loading = true;
    this.error = null;
    try {
      const items = await invoke<MediaEntry[]>("scan_folder", { folder: folderPath });
      this.folder = folderPath;
      this.allItems = items;
      const remembered = this.history.find((h) => h.folder === folderPath)?.last_item;
      const resolved = focusPath ?? remembered ?? undefined;
      this.currentPath = (resolved && items.some((i) => i.path === resolved))
        ? resolved
        : items[0]?.path ?? null;
      this.clearSelection();
      this.zoom = 1;
      this.siblingFolders = await invoke<string[]>("sibling_folders", { folder: folderPath });
      await this.recordHistory(folderPath, this.currentPath);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async openPath(path: string) {
    const parent = await invoke<string | null>("parent_dir", { path });
    if (parent) {
      await this.openFolder(parent, path);
      this.view = "viewer";
    }
  }

  /** Loads recently opened folders from disk, most-recent first. */
  async loadHistory() {
    try {
      this.history = await invoke<HistoryEntry[]>("get_history");
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Reopens the most recently viewed folder at the last-viewed item. Returns false if there's no history or it no longer exists. */
  async resumeLastSession(): Promise<boolean> {
    await this.loadHistory();
    const last = this.history[0];
    if (!last) return false;
    await this.openFolder(last.folder, last.last_item ?? undefined);
    if (this.error) {
      this.error = null;
      this.folder = null;
      this.allItems = [];
      this.currentPath = null;
      return false;
    }
    this.view = "viewer";
    return true;
  }

  async openHistoryEntry(entry: HistoryEntry) {
    await this.openFolder(entry.folder, entry.last_item ?? undefined);
    this.view = "viewer";
  }

  private async recordHistory(folder: string, lastItem: string | null) {
    try {
      this.history = await invoke<HistoryEntry[]>("record_history", { folder, lastItem });
    } catch {
      // Non-fatal: history is a convenience, not core functionality.
    }
  }

  /** Debounces history writes so rapid arrow-key navigation doesn't hammer disk IO. */
  private scheduleHistoryUpdate() {
    if (!this.folder) return;
    clearTimeout(this.historyTimer);
    const folder = this.folder;
    const path = this.currentPath;
    this.historyTimer = setTimeout(() => {
      this.recordHistory(folder, path);
    }, HISTORY_DEBOUNCE_MS);
  }

  requestClearCache() {
    this.pendingClearCache = true;
  }

  cancelClearCache() {
    this.pendingClearCache = false;
  }

  async confirmClearCache() {
    this.pendingClearCache = false;
    try {
      await invoke("clear_history");
      await invoke("clear_trash");
      this.history = [];
      this.undoStack = [];
    } catch (e) {
      this.error = String(e);
    }
  }

  select(path: string) {
    this.currentPath = path;
    this.scheduleHistoryUpdate();
  }

  next() {
    const items = this.filteredItems;
    if (items.length === 0) return;
    const idx = this.currentIndex;
    const nextIdx = idx < 0 ? 0 : (idx + 1) % items.length;
    this.currentPath = items[nextIdx].path;
    this.scheduleHistoryUpdate();
  }

  prev() {
    const items = this.filteredItems;
    if (items.length === 0) return;
    const idx = this.currentIndex;
    const prevIdx = idx < 0 ? 0 : (idx - 1 + items.length) % items.length;
    this.currentPath = items[prevIdx].path;
    this.scheduleHistoryUpdate();
  }

  /** Steps to the adjacent sibling folder (ArrowUp/ArrowDown), wrapping around. */
  private async stepFolder(direction: 1 | -1) {
    if (!this.folder || this.siblingFolders.length === 0) return;
    const idx = this.siblingFolders.indexOf(this.folder);
    if (idx < 0) return;
    const nextIdx = (idx + direction + this.siblingFolders.length) % this.siblingFolders.length;
    await this.openFolder(this.siblingFolders[nextIdx]);
  }

  nextFolder() {
    return this.stepFolder(1);
  }

  prevFolder() {
    return this.stepFolder(-1);
  }

  zoomIn() {
    this.zoom = Math.min(MAX_ZOOM, Math.round((this.zoom + ZOOM_STEP) * 100) / 100);
  }

  zoomOut() {
    this.zoom = Math.max(MIN_ZOOM, Math.round((this.zoom - ZOOM_STEP) * 100) / 100);
  }

  resetZoom() {
    this.zoom = 1;
  }

  clearSelection() {
    this.selectedPaths = new Set();
    this.selectionAnchor = null;
  }

  selectAll() {
    this.selectedPaths = new Set(this.filteredItems.map((i) => i.path));
  }

  toggleSelect(path: string) {
    const next = new Set(this.selectedPaths);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    this.selectedPaths = next;
    this.selectionAnchor = path;
  }

  /** Adds the current item to the selection tray (Space key). */
  addCurrentToSelection() {
    if (!this.currentPath || this.selectedPaths.has(this.currentPath)) return;
    const next = new Set(this.selectedPaths);
    next.add(this.currentPath);
    this.selectedPaths = next;
    this.selectionAnchor = this.currentPath;
  }

  removeFromSelection(path: string) {
    if (!this.selectedPaths.has(path)) return;
    const next = new Set(this.selectedPaths);
    next.delete(path);
    this.selectedPaths = next;
  }

  selectRange(path: string) {
    const items = this.filteredItems;
    const anchor = this.selectionAnchor ?? this.currentPath ?? path;
    const from = items.findIndex((i) => i.path === anchor);
    const to = items.findIndex((i) => i.path === path);
    if (from < 0 || to < 0) {
      this.toggleSelect(path);
      return;
    }
    const [lo, hi] = from < to ? [from, to] : [to, from];
    const next = new Set(this.selectedPaths);
    for (let i = lo; i <= hi; i++) next.add(items[i].path);
    this.selectedPaths = next;
  }

  private saveFavorites() {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(FAVORITES_KEY, JSON.stringify([...this.favorites]));
  }

  isFavorite(path: string) {
    return this.favorites.has(path);
  }

  /** Toggles favorite status for the given path, or the current item if omitted. */
  toggleFavorite(path?: string) {
    const target = path ?? this.currentPath;
    if (!target) return;
    const next = new Set(this.favorites);
    if (next.has(target)) next.delete(target);
    else next.add(target);
    this.favorites = next;
    this.saveFavorites();
  }

  async copyCurrentPath() {
    if (!this.current) return;
    await writeText(this.current.path);
  }

  /** Opens the confirm-delete flow for the given paths, or the current selection/item. */
  requestDelete(paths?: string[]) {
    const targets = paths ?? (this.selectedPaths.size > 0 ? [...this.selectedPaths] : this.current ? [this.current.path] : []);
    if (targets.length === 0) return;
    if (this.skipDeleteConfirm) {
      this.performDelete(targets);
    } else {
      this.pendingDelete = targets;
    }
  }

  cancelDelete() {
    this.pendingDelete = null;
  }

  async confirmDelete(dontAskAgain: boolean) {
    const targets = this.pendingDelete;
    this.pendingDelete = null;
    if (!targets) return;
    if (dontAskAgain) {
      this.skipDeleteConfirm = true;
      localStorage.setItem(SKIP_DELETE_CONFIRM_KEY, "true");
    }
    await this.performDelete(targets);
  }

  private async performDelete(paths: string[]) {
    try {
      const batch = await invoke<TrashEntry[]>("soft_delete", { paths });
      this.undoStack = [...this.undoStack, batch].slice(-MAX_UNDO_BATCHES);
      const removed = new Set(paths);
      const wasCurrentRemoved = this.currentPath !== null && removed.has(this.currentPath);
      const prevItems = this.filteredItems;
      const prevIndex = this.currentIndex;

      this.allItems = this.allItems.filter((i) => !removed.has(i.path));

      if (wasCurrentRemoved) {
        // Prefer the nearest surviving item before the deleted one; fall back to the nearest after.
        let fallback: string | null = null;
        for (let i = prevIndex - 1; i >= 0; i--) {
          if (!removed.has(prevItems[i].path)) {
            fallback = prevItems[i].path;
            break;
          }
        }
        if (!fallback) {
          for (let i = prevIndex + 1; i < prevItems.length; i++) {
            if (!removed.has(prevItems[i].path)) {
              fallback = prevItems[i].path;
              break;
            }
          }
        }
        this.currentPath = fallback;
      }

      const nextSelected = new Set(this.selectedPaths);
      for (const p of paths) nextSelected.delete(p);
      this.selectedPaths = nextSelected;

      if (paths.some((p) => this.favorites.has(p))) {
        const nextFavorites = new Set(this.favorites);
        for (const p of paths) nextFavorites.delete(p);
        this.favorites = nextFavorites;
        this.saveFavorites();
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Restores the most recently deleted batch of files. */
  async undo() {
    const batch = this.undoStack.at(-1);
    if (!batch) return;
    this.undoStack = this.undoStack.slice(0, -1);
    try {
      await invoke("restore_paths", { originalPaths: batch.map((e) => e.original_path) });
      await this.refresh();
      const restored = batch[0]?.original_path;
      if (restored && this.allItems.some((i) => i.path === restored)) {
        this.currentPath = restored;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Re-scans the current folder, e.g. after restoring files from the trash. */
  async refresh() {
    if (!this.folder) return;
    try {
      this.allItems = await invoke<MediaEntry[]>("scan_folder", { folder: this.folder });
    } catch (e) {
      this.error = String(e);
    }
  }

  async findDuplicates() {
    if (!this.folder) return;
    this.findingDuplicates = true;
    this.error = null;
    try {
      this.duplicateGroups = await invoke<DuplicateGroup[]>("find_duplicates", {
        folder: this.folder,
      });
      this.showDuplicates = true;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.findingDuplicates = false;
    }
  }
}

export const gallery = new GalleryStore();
