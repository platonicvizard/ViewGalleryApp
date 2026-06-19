<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { gallery } from "./gallery.svelte";

  let { onClose }: { onClose: () => void } = $props();

  // Pre-check every duplicate but the first in each group, so "keep one copy" is the default.
  let checked = $state<Set<string>>(
    new Set(gallery.duplicateGroups.flatMap((g) => g.paths.slice(1))),
  );

  function toggle(path: string) {
    const next = new Set(checked);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    checked = next;
  }

  function basename(path: string) {
    return path.split(/[/\\]/).pop() ?? path;
  }

  function isImage(path: string) {
    return gallery.allItems.find((i) => i.path === path)?.kind !== "video";
  }

  function deleteChecked() {
    const targets = [...checked];
    if (targets.length === 0) return;
    onClose();
    gallery.requestDelete(targets);
  }
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
  role="presentation"
  onclick={onClose}
>
  <div
    class="w-[480px] max-h-[80vh] overflow-y-auto rounded-xl bg-surface border border-white/10 p-5 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold text-white">
        Duplicate files
        <span class="text-white/40 font-normal">({gallery.duplicateGroups.length} groups)</span>
      </h2>
      <button
        class="w-6 h-6 rounded-md bg-white/5 hover:bg-white/10 text-xs flex items-center justify-center transition"
        onclick={onClose}
        aria-label="Close"
      >
        ✕
      </button>
    </div>

    {#if gallery.duplicateGroups.length === 0}
      <p class="mt-4 text-xs text-white/50">No exact duplicates found in this folder.</p>
    {:else}
      <p class="mt-2 text-xs text-white/50">
        Checked files will be moved to the trash. The first copy in each group is unchecked by default.
      </p>
      <div class="mt-3 space-y-4">
        {#each gallery.duplicateGroups as group, gi (group.hash)}
          <div>
            <div class="text-[11px] text-white/40 mb-1.5">Group {gi + 1} · {group.paths.length} copies</div>
            <div class="space-y-1.5">
              {#each group.paths as path (path)}
                <label class="flex items-center gap-2.5 px-2 py-1.5 rounded-md bg-white/5">
                  <input
                    type="checkbox"
                    class="accent-accent"
                    checked={checked.has(path)}
                    onchange={() => toggle(path)}
                  />
                  {#if isImage(path)}
                    <img
                      src={convertFileSrc(path)}
                      alt=""
                      class="w-8 h-8 rounded object-cover bg-bg shrink-0"
                    />
                  {:else}
                    <span class="w-8 h-8 rounded bg-bg shrink-0 flex items-center justify-center text-white/50 text-xs">▶</span>
                  {/if}
                  <span class="text-xs text-white/70 truncate">{basename(path)}</span>
                </label>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="mt-5 flex justify-end gap-2">
        <button
          class="px-3 py-1.5 rounded-md bg-white/5 text-sm font-medium hover:bg-white/10 transition"
          onclick={onClose}
        >
          Close
        </button>
        <button
          class="px-3 py-1.5 rounded-md bg-red-600 text-white text-sm font-medium hover:brightness-110 transition disabled:opacity-40"
          disabled={checked.size === 0}
          onclick={deleteChecked}
        >
          Move {checked.size} to Trash
        </button>
      </div>
    {/if}
  </div>
</div>
