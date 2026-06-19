<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { gallery } from "./gallery.svelte";
  import type { KindFilter, SortKey } from "./types";

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await gallery.openFolder(selected);
    }
  }

  async function pickFile() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Media",
          extensions: [
            "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif",
            "avif", "heic", "heif", "svg",
            "mp4", "mov", "webm", "mkv", "avi", "m4v", "wmv", "flv", "mpg", "mpeg",
          ],
        },
      ],
    });
    if (typeof selected === "string") {
      await gallery.openPath(selected);
    }
  }

  const kinds: { value: KindFilter; label: string }[] = [
    { value: "all", label: "All" },
    { value: "image", label: "Images" },
    { value: "video", label: "Videos" },
  ];

  const sortOptions: { value: SortKey; label: string }[] = [
    { value: "name", label: "Name" },
    { value: "date", label: "Date" },
    { value: "size", label: "Size" },
  ];

  function toggleSortDir() {
    gallery.sortDir = gallery.sortDir === "asc" ? "desc" : "asc";
  }
</script>

<div class="flex items-center gap-3 px-4 py-2.5 bg-surface/80 border-b border-white/5 backdrop-blur">
  <div class="flex items-center gap-2">
    <button
      class="px-3 py-1.5 rounded-md bg-accent text-white text-sm font-medium hover:brightness-110 transition"
      onclick={pickFolder}
    >
      Open Folder
    </button>
    <button
      class="px-3 py-1.5 rounded-md bg-white/5 text-sm font-medium hover:bg-white/10 transition"
      onclick={pickFile}
    >
      Open File
    </button>
  </div>

  <div class="flex-1 flex items-center gap-2 max-w-md">
    <input
      type="text"
      placeholder="Filter by name…"
      class="w-full px-3 py-1.5 rounded-md bg-white/5 text-sm placeholder:text-white/35 outline-none focus:ring-1 focus:ring-accent"
      bind:value={gallery.searchText}
    />
  </div>

  <div class="flex items-center gap-1">
    <select
      class="px-2 py-1.5 rounded-md bg-white/5 text-xs font-medium outline-none focus:ring-1 focus:ring-accent appearance-none cursor-pointer"
      bind:value={gallery.sortKey}
    >
      {#each sortOptions as s}
        <option value={s.value}>Sort: {s.label}</option>
      {/each}
    </select>
    <button
      class="w-7 h-7 rounded-md bg-white/5 hover:bg-white/10 text-xs flex items-center justify-center transition"
      onclick={toggleSortDir}
      title={gallery.sortDir === "asc" ? "Ascending" : "Descending"}
      aria-label="Toggle sort direction"
    >
      {gallery.sortDir === "asc" ? "↑" : "↓"}
    </button>
  </div>

  <div class="flex items-center gap-1 bg-white/5 rounded-md p-0.5">
    {#each kinds as k}
      <button
        class="px-2.5 py-1 rounded text-xs font-medium transition {gallery.kindFilter === k.value
          ? 'bg-accent text-white'
          : 'text-white/60 hover:text-white'}"
        onclick={() => (gallery.kindFilter = k.value)}
      >
        {k.label}
      </button>
    {/each}
  </div>

  <div class="flex items-center gap-1 bg-white/5 rounded-md p-0.5">
    <button
      class="px-2.5 py-1 rounded text-xs font-medium transition {gallery.view === 'viewer'
        ? 'bg-accent text-white'
        : 'text-white/60 hover:text-white'}"
      onclick={() => (gallery.view = "viewer")}
    >
      Viewer
    </button>
    <button
      class="px-2.5 py-1 rounded text-xs font-medium transition {gallery.view === 'grid'
        ? 'bg-accent text-white'
        : 'text-white/60 hover:text-white'}"
      onclick={() => (gallery.view = "grid")}
    >
      Grid
    </button>
  </div>

  {#if gallery.filteredItems.length > 0}
    <div class="text-xs text-white/40 tabular-nums whitespace-nowrap">
      {gallery.currentIndex + 1} / {gallery.filteredItems.length}
    </div>
  {/if}
</div>
