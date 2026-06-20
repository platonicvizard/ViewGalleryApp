<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  const GAP = 10;
  const HEADER_H = 30;
  const GROUP_GAP = 16; // matches the `space-y-4` gap between group sections

  let containerEl: HTMLDivElement;
  let containerWidth = $state(0);
  let containerHeight = $state(0);
  let scrollTop = $state(0);

  // Shrink tiles on narrow windows so the grid stays usable on smaller screens.
  let TILE = $derived(containerWidth > 0 && containerWidth < 480 ? 110 : containerWidth < 700 ? 130 : 160);

  let items = $derived(gallery.filteredItems);
  let cols = $derived(Math.max(1, Math.floor((containerWidth + GAP) / (TILE + GAP))));
  let rows = $derived(Math.ceil(items.length / cols));
  let rowHeight = $derived(TILE + GAP);
  let totalHeight = $derived(rows * rowHeight);

  // Only render the rows currently in (or near) the viewport so huge folders
  // stay smooth instead of mounting thousands of <img> nodes at once.
  let firstRow = $derived(Math.max(0, Math.floor(scrollTop / rowHeight) - 2));
  let lastRow = $derived(
    Math.min(rows, Math.ceil((scrollTop + containerHeight) / rowHeight) + 2),
  );
  let visibleItems = $derived.by(() => {
    const start = firstRow * cols;
    const end = Math.min(items.length, lastRow * cols);
    const out: { item: (typeof items)[number]; index: number }[] = [];
    for (let i = start; i < end; i++) out.push({ item: items[i], index: i });
    return out;
  });

  function tilePos(index: number) {
    const row = Math.floor(index / cols);
    const col = index % cols;
    return { top: row * rowHeight, left: col * (TILE + GAP) };
  }

  function open(path: string) {
    gallery.select(path);
    gallery.view = "viewer";
  }

  function handleClick(e: MouseEvent, path: string) {
    if (e.shiftKey) {
      gallery.selectRange(path);
      gallery.select(path);
    } else if (e.metaKey || e.ctrlKey) {
      gallery.toggleSelect(path);
      gallery.select(path);
    } else {
      gallery.clearSelection();
      open(path);
    }
  }

  function toggleFavorite(e: MouseEvent, path: string) {
    e.stopPropagation();
    gallery.toggleFavorite(path);
  }

  let groups = $derived.by(() => {
    if (gallery.groupBy === "none") return null;
    const map = new Map<string, typeof items>();
    for (const item of items) {
      const key =
        gallery.groupBy === "date"
          ? new Date(item.modified).toLocaleDateString()
          : item.ext.toUpperCase();
      const bucket = map.get(key);
      if (bucket) bucket.push(item);
      else map.set(key, [item]);
    }
    return [...map.entries()];
  });

  function tileClass(path: string) {
    const selected = gallery.selectedPaths.has(path);
    const current = path === gallery.currentPath;
    if (selected) return "border-accent ring-2 ring-accent/60";
    if (current) return "border-accent";
    return "border-transparent hover:border-white/20";
  }

  // Windows each group's items to just the rows near the viewport, same idea as the
  // ungrouped grid, so grouping huge folders doesn't mount thousands of DOM nodes at once.
  let groupLayout = $derived.by(() => {
    if (!groups) return [];
    let offset = 0;
    return groups.map(([label, groupItems]) => {
      const groupRows = Math.ceil(groupItems.length / cols);
      const itemsTop = offset + HEADER_H;
      const groupHeight = HEADER_H + groupRows * rowHeight;
      offset += groupHeight + GROUP_GAP;

      const firstVisibleRow = Math.max(0, Math.floor((scrollTop - itemsTop) / rowHeight) - 2);
      const lastVisibleRow = Math.min(
        groupRows,
        Math.ceil((scrollTop + containerHeight - itemsTop) / rowHeight) + 2,
      );
      const start = firstVisibleRow * cols;
      const end = Math.min(groupItems.length, lastVisibleRow * cols);

      return {
        label,
        count: groupItems.length,
        visibleItems: groupItems.slice(start, end),
        topSpacer: firstVisibleRow * rowHeight,
        bottomSpacer: (groupRows - lastVisibleRow) * rowHeight,
      };
    });
  });
</script>

{#snippet tile(item: (typeof items)[number])}
  <div
    role="button"
    tabindex="0"
    class="rounded-lg overflow-hidden border-2 transition relative {tileClass(item.path)}"
    style="width: {TILE}px; height: {TILE}px;"
    onclick={(e) => handleClick(e, item.path)}
    onkeydown={(e) => {
      if (e.key === "Enter") handleClick(e as unknown as MouseEvent, item.path);
    }}
  >
    {#if item.kind === "image"}
      <img
        src={convertFileSrc(item.path)}
        alt={item.name}
        loading="lazy"
        class="w-full h-full object-cover bg-surface"
      />
    {:else}
      <div class="relative w-full h-full bg-surface">
        <video
          src={convertFileSrc(item.path)}
          class="w-full h-full object-cover"
          muted
          preload="none"
        ></video>
        <span class="absolute inset-0 flex items-center justify-center text-white text-lg">▶</span>
      </div>
    {/if}
    <button
      class="absolute top-1 right-1 w-5 h-5 rounded-full bg-black/50 hover:bg-black/70 flex items-center justify-center text-xs transition {gallery.isFavorite(
        item.path,
      )
        ? 'text-yellow-400'
        : 'text-white/60'}"
      onclick={(e) => toggleFavorite(e, item.path)}
      title="Toggle favorite"
      aria-label="Toggle favorite"
    >
      {gallery.isFavorite(item.path) ? "★" : "☆"}
    </button>
    <span
      class="absolute bottom-0 inset-x-0 px-1.5 py-1 bg-black/60 text-[10px] text-white/80 truncate"
      >{item.name}</span
    >
  </div>
{/snippet}

{#if items.length === 0}
  <div class="flex-1 flex items-center justify-center text-white/40 text-sm">
    No media to show
  </div>
{:else if groups}
  <div
    bind:this={containerEl}
    bind:clientWidth={containerWidth}
    bind:clientHeight={containerHeight}
    onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
    class="flex-1 overflow-y-auto p-3 space-y-4"
  >
    {#each groupLayout as g (g.label)}
      <div>
        <div class="sticky top-0 z-10 px-1 py-1.5 text-xs font-medium text-white/60 bg-bg/90 backdrop-blur">
          {g.label} <span class="text-white/30">({g.count})</span>
        </div>
        {#if g.topSpacer > 0}<div style="height: {g.topSpacer}px;"></div>{/if}
        <div class="flex flex-wrap gap-2.5">
          {#each g.visibleItems as item (item.path)}
            {@render tile(item)}
          {/each}
        </div>
        {#if g.bottomSpacer > 0}<div style="height: {g.bottomSpacer}px;"></div>{/if}
      </div>
    {/each}
  </div>
{:else}
  <div
    bind:this={containerEl}
    bind:clientWidth={containerWidth}
    bind:clientHeight={containerHeight}
    onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
    class="flex-1 overflow-y-auto p-3"
  >
    <div class="relative" style="height: {totalHeight}px;">
      {#each visibleItems as { item, index } (item.path)}
        {@const pos = tilePos(index)}
        <div class="absolute" style="top: {pos.top}px; left: {pos.left}px;">
          {@render tile(item)}
        </div>
      {/each}
    </div>
  </div>
{/if}
