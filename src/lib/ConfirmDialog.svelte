<script lang="ts">
  let {
    title,
    message,
    confirmLabel = "Confirm",
    showDontAskAgain = true,
    onConfirm,
    onCancel,
  }: {
    title: string;
    message?: string;
    confirmLabel?: string;
    showDontAskAgain?: boolean;
    onConfirm: (dontAskAgain: boolean) => void;
    onCancel: () => void;
  } = $props();

  let dontAskAgain = $state(false);
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
  role="presentation"
  onclick={onCancel}
>
  <div
    class="w-[360px] rounded-xl bg-surface border border-white/10 p-5 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <h2 class="text-sm font-semibold text-white">{title}</h2>
    {#if message}
      <p class="mt-2 text-xs text-white/60">{message}</p>
    {/if}

    {#if showDontAskAgain}
      <label class="mt-4 flex items-center gap-2 text-xs text-white/70 cursor-pointer">
        <input type="checkbox" bind:checked={dontAskAgain} class="accent-accent" />
        Don't ask me again
      </label>
    {/if}

    <div class="mt-5 flex justify-end gap-2">
      <button
        class="px-3 py-1.5 rounded-md bg-white/5 text-sm font-medium hover:bg-white/10 transition"
        onclick={onCancel}
      >
        Cancel
      </button>
      <button
        class="px-3 py-1.5 rounded-md bg-red-600 text-white text-sm font-medium hover:brightness-110 transition"
        onclick={() => onConfirm(dontAskAgain)}
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
