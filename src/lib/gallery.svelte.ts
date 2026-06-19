import { invoke } from "@tauri-apps/api/core";
import type { KindFilter, MediaEntry, SortDir, SortKey } from "./types";

const nameCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

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

  filteredItems = $derived.by(() => {
    const q = this.searchText.trim().toLowerCase();
    const filtered = this.allItems.filter((item) => {
      if (this.kindFilter !== "all" && item.kind !== this.kindFilter) return false;
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

  async openFolder(folderPath: string, focusPath?: string) {
    this.loading = true;
    this.error = null;
    try {
      const items = await invoke<MediaEntry[]>("scan_folder", { folder: folderPath });
      this.folder = folderPath;
      this.allItems = items;
      this.currentPath = focusPath ?? items[0]?.path ?? null;
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

  select(path: string) {
    this.currentPath = path;
  }

  next() {
    const items = this.filteredItems;
    if (items.length === 0) return;
    const idx = this.currentIndex;
    const nextIdx = idx < 0 ? 0 : (idx + 1) % items.length;
    this.currentPath = items[nextIdx].path;
  }

  prev() {
    const items = this.filteredItems;
    if (items.length === 0) return;
    const idx = this.currentIndex;
    const prevIdx = idx < 0 ? 0 : (idx - 1 + items.length) % items.length;
    this.currentPath = items[prevIdx].path;
  }
}

export const gallery = new GalleryStore();
