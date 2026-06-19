<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();

  const shortcuts: { keys: string; description: string }[] = [
    { keys: "← / →", description: "Previous / next file" },
    { keys: "↑ / ↓", description: "Previous / next folder" },
    { keys: "+ / -", description: "Zoom in / out" },
    { keys: "0", description: "Reset zoom" },
    { keys: "Esc", description: "Back to grid / close dialog" },
    { keys: "Delete", description: "Move selected file(s) to trash" },
    { keys: "⌘/Ctrl + Z", description: "Undo last delete" },
    { keys: "⌘/Ctrl + A", description: "Select all (grid view)" },
    { keys: "⌘/Ctrl + C", description: "Copy current file path" },
    { keys: "⌘/Ctrl + D", description: "Find duplicate files" },
    { keys: "⌘/Ctrl + G", description: "Cycle grouping (none / date / type)" },
    { keys: "Shift + click", description: "Select range (grid view)" },
    { keys: "⌘/Ctrl + click", description: "Toggle item selection (grid view)" },
    { keys: "?", description: "Show this help" },
  ];
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
  role="presentation"
  onclick={onClose}
>
  <div
    class="w-[420px] max-h-[80vh] overflow-y-auto rounded-xl bg-surface border border-white/10 p-5 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold text-white">Keyboard shortcuts</h2>
      <button
        class="w-6 h-6 rounded-md bg-white/5 hover:bg-white/10 text-xs flex items-center justify-center transition"
        onclick={onClose}
        aria-label="Close"
      >
        ✕
      </button>
    </div>

    <div class="mt-3 divide-y divide-white/5">
      {#each shortcuts as s}
        <div class="flex items-center justify-between gap-4 py-2">
          <span class="text-xs text-white/60">{s.description}</span>
          <kbd class="px-2 py-1 rounded-md bg-white/10 text-[11px] font-mono text-white/80 whitespace-nowrap">
            {s.keys}
          </kbd>
        </div>
      {/each}
    </div>
  </div>
</div>
