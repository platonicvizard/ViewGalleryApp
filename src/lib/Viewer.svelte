<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  let current = $derived(gallery.current);
  let items = $derived(gallery.filteredItems);
  let idx = $derived(gallery.currentIndex);

  // Preload the neighbouring images so arrow-key navigation feels instant.
  $effect(() => {
    if (idx < 0) return;
    for (const offset of [-1, 1]) {
      const neighbour = items[(idx + offset + items.length) % items.length];
      if (neighbour?.kind === "image") {
        const img = new Image();
        img.src = convertFileSrc(neighbour.path);
      }
    }
  });

  // Reset zoom whenever the displayed item changes.
  $effect(() => {
    current;
    gallery.resetZoom();
  });
</script>

<div class="relative flex-1 flex items-center justify-center overflow-hidden bg-bg">
  {#if !current}
    <div class="text-white/40 text-sm">No media selected</div>
  {:else if current.kind === "image"}
    {#key current.path}
      <img
        src={convertFileSrc(current.path)}
        alt={current.name}
        class="max-h-full max-w-full object-contain select-none transition-transform"
        style="transform: scale({gallery.zoom});"
        draggable="false"
      />
    {/key}
  {:else}
    {#key current.path}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        src={convertFileSrc(current.path)}
        class="max-h-full max-w-full"
        controls
        autoplay
        playsinline
      ></video>
    {/key}
  {/if}

  {#if items.length > 1}
    <button
      class="absolute left-3 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-black/40 hover:bg-black/60 text-white flex items-center justify-center transition"
      onclick={() => gallery.prev()}
      aria-label="Previous"
    >
      ‹
    </button>
    <button
      class="absolute right-3 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-black/40 hover:bg-black/60 text-white flex items-center justify-center transition"
      onclick={() => gallery.next()}
      aria-label="Next"
    >
      ›
    </button>
  {/if}

  {#if current}
    <div
      class="absolute bottom-3 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full bg-black/40 text-xs text-white/80 max-w-[80%] truncate"
    >
      {current.name}
    </div>
  {/if}

  {#if current?.kind === "image" && gallery.zoom !== 1}
    <div
      class="absolute bottom-3 right-3 px-2 py-1 rounded-full bg-black/40 text-xs text-white/80 tabular-nums"
    >
      {Math.round(gallery.zoom * 100)}%
    </div>
  {/if}
</div>
