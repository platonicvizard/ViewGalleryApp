<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  // Selection order isn't tracked explicitly; filteredItems order is stable and
  // cheap to filter down to just the selected paths for display.
  let items = $derived(gallery.filteredItems.filter((i) => gallery.selectedPaths.has(i.path)));
</script>

{#if items.length > 0}
  <div
    class="flex items-center gap-2 px-2 sm:px-3 py-1.5 bg-surface/80 border-t border-white/10 backdrop-blur"
  >
    <span class="shrink-0 text-[11px] text-white/50 tabular-nums px-1">
      {items.length} selected
    </span>
    <div class="flex-1 flex gap-1.5 overflow-x-auto py-0.5">
      {#each items as item (item.path)}
        <button
          class="relative shrink-0 w-10 h-10 sm:w-12 sm:h-12 rounded-md overflow-hidden border-2 transition {item.path ===
          gallery.currentPath
            ? 'border-accent'
            : 'border-transparent opacity-80 hover:opacity-100'}"
          title={item.name}
          onclick={() => gallery.select(item.path)}
        >
          {#if item.kind === "image"}
            <img
              src={convertFileSrc(item.path)}
              alt={item.name}
              loading="lazy"
              class="w-full h-full object-cover bg-black/30"
            />
          {:else}
            <video
              src={convertFileSrc(item.path)}
              class="w-full h-full object-cover bg-black/30"
              muted
              preload="none"
            ></video>
          {/if}
          <span
            class="absolute -top-0.5 -right-0.5 w-4 h-4 rounded-full bg-black/70 hover:bg-red-600 text-white text-[10px] flex items-center justify-center"
            onclick={(e) => {
              e.stopPropagation();
              gallery.removeFromSelection(item.path);
            }}
            onkeydown={(e) => {
              if (e.key === "Enter") {
                e.stopPropagation();
                gallery.removeFromSelection(item.path);
              }
            }}
            role="button"
            tabindex="0"
            aria-label="Remove from selection"
          >
            ×
          </span>
        </button>
      {/each}
    </div>
    <button
      class="shrink-0 px-2 py-1 rounded-md bg-white/10 hover:bg-white/20 text-[11px] font-medium transition"
      onclick={() => gallery.clearSelection()}
    >
      Clear
    </button>
  </div>
{/if}
