<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  const TILE = 160;
  const GAP = 10;

  let containerEl: HTMLDivElement;
  let containerWidth = $state(0);
  let containerHeight = $state(0);
  let scrollTop = $state(0);

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
</script>

<div
  bind:this={containerEl}
  bind:clientWidth={containerWidth}
  bind:clientHeight={containerHeight}
  onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
  class="flex-1 overflow-y-auto p-3"
>
  {#if items.length === 0}
    <div class="text-white/40 text-sm text-center mt-10">No media to show</div>
  {:else}
    <div class="relative" style="height: {totalHeight}px;">
      {#each visibleItems as { item, index } (item.path)}
        {@const pos = tilePos(index)}
        <button
          class="absolute rounded-lg overflow-hidden border-2 transition {item.path ===
          gallery.currentPath
            ? 'border-accent'
            : 'border-transparent hover:border-white/20'}"
          style="top: {pos.top}px; left: {pos.left}px; width: {TILE}px; height: {TILE}px;"
          onclick={() => open(item.path)}
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
              <video src={convertFileSrc(item.path)} class="w-full h-full object-cover" muted
              ></video>
              <span class="absolute inset-0 flex items-center justify-center text-white text-lg"
                >▶</span
              >
            </div>
          {/if}
          <span
            class="absolute bottom-0 inset-x-0 px-1.5 py-1 bg-black/60 text-[10px] text-white/80 truncate"
            >{item.name}</span
          >
        </button>
      {/each}
    </div>
  {/if}
</div>
