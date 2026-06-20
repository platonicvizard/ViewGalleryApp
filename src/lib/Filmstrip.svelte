<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  let items = $derived(gallery.filteredItems);
  let strip: HTMLDivElement = $state()!;

  $effect(() => {
    const path = gallery.currentPath;
    if (!path || !strip) return;
    const el = strip.querySelector<HTMLElement>(`[data-path="${CSS.escape(path)}"]`);
    el?.scrollIntoView({ block: "nearest", inline: "center", behavior: "smooth" });
  });
</script>

{#if items.length > 0}
  <div
    bind:this={strip}
    class="flex gap-1.5 sm:gap-2 px-2 sm:px-3 py-1.5 sm:py-2 overflow-x-auto bg-surface/60 border-t border-white/5"
  >
    {#each items as item (item.path)}
      <button
        data-path={item.path}
        class="relative shrink-0 w-12 h-12 sm:w-16 sm:h-16 rounded-md overflow-hidden border-2 transition {item.path ===
        gallery.currentPath
          ? 'border-accent'
          : 'border-transparent opacity-70 hover:opacity-100'}"
        onclick={() => gallery.select(item.path)}
      >
        {#if item.kind === "image"}
          <img
            src={convertFileSrc(item.path)}
            alt={item.name}
            loading="lazy"
            class="w-full h-full object-cover"
          />
        {:else}
          <video
            src={convertFileSrc(item.path)}
            class="w-full h-full object-cover"
            muted
            preload="none"
          ></video>
          <span
            class="absolute inset-0 flex items-center justify-center bg-black/30 text-white text-xs"
            >▶</span
          >
        {/if}
      </button>
    {/each}
  </div>
{/if}
