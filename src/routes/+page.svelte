<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { gallery } from "$lib/gallery.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import Viewer from "$lib/Viewer.svelte";
  import Filmstrip from "$lib/Filmstrip.svelte";
  import Grid from "$lib/Grid.svelte";
  import SelectionTray from "$lib/SelectionTray.svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import HelpModal from "$lib/HelpModal.svelte";
  import DuplicatesModal from "$lib/DuplicatesModal.svelte";

  function onKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT") return;

    // Modals take over Escape first; other shortcuts are suppressed while one is open.
    if (gallery.showHelp || gallery.showDuplicates || gallery.pendingDelete || gallery.pendingClearCache) {
      if (e.key === "Escape") {
        e.preventDefault();
        gallery.showHelp = false;
        gallery.showDuplicates = false;
        gallery.cancelDelete();
        gallery.cancelClearCache();
      }
      return;
    }

    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key.toLowerCase() === "z") {
      e.preventDefault();
      gallery.undo();
      return;
    }
    if (mod && e.key.toLowerCase() === "a") {
      e.preventDefault();
      gallery.selectAll();
      return;
    }
    if (mod && e.key.toLowerCase() === "c") {
      e.preventDefault();
      gallery.copyCurrentPath();
      return;
    }
    if (mod && e.key.toLowerCase() === "d") {
      e.preventDefault();
      gallery.findDuplicates();
      return;
    }
    if (mod && e.key.toLowerCase() === "g") {
      e.preventDefault();
      const order: typeof gallery.groupBy[] = ["none", "date", "type"];
      gallery.groupBy = order[(order.indexOf(gallery.groupBy) + 1) % order.length];
      return;
    }
    if (e.key === "?") {
      e.preventDefault();
      gallery.showHelp = true;
      return;
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      gallery.requestDelete();
      return;
    }
    if (e.code === "Space") {
      e.preventDefault();
      gallery.addCurrentToSelection();
      return;
    }
    if (e.key.toLowerCase() === "f") {
      e.preventDefault();
      gallery.toggleFavorite();
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      gallery.prevFolder();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      gallery.nextFolder();
      return;
    }

    if (gallery.view !== "viewer") return;

    if (e.key === "ArrowRight") {
      e.preventDefault();
      gallery.next();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      gallery.prev();
    } else if (e.key === "+" || e.key === "=") {
      e.preventDefault();
      gallery.zoomIn();
    } else if (e.key === "-" || e.key === "_") {
      e.preventDefault();
      gallery.zoomOut();
    } else if (e.key === "0") {
      e.preventDefault();
      gallery.resetZoom();
    } else if (e.key === "Escape") {
      gallery.view = "grid";
    }
  }

  onMount(() => {
    const win = getCurrentWebviewWindow();
    let unlistenOpen: (() => void) | undefined;
    let unlistenDrop: (() => void) | undefined;

    (async () => {
      const initial = await invoke<string | null>("take_initial_path");
      if (initial) {
        await gallery.loadHistory();
        await gallery.openPath(initial);
      } else {
        await gallery.resumeLastSession();
      }

      unlistenOpen = await win.listen<string>("open-path", (event) => {
        gallery.openPath(event.payload);
      });

      unlistenDrop = await win.onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          const first = event.payload.paths[0];
          if (first) gallery.openPath(first);
        }
      });
    })();

    return () => {
      unlistenOpen?.();
      unlistenDrop?.();
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<main class="h-screen w-screen flex flex-col">
  <Toolbar />

  {#if gallery.folder}
    <button
      class="block w-full px-4 py-1 text-xs text-white/40 truncate text-left bg-surface/40 border-b border-white/5 hover:text-white/70 transition"
      title="Click to copy path"
      onclick={() => gallery.copyCurrentPath()}
    >
      {gallery.folder}{#if gallery.current}<span class="text-white/25">/</span>{gallery.current.name}{/if}
    </button>
  {/if}

  {#if gallery.error}
    <div class="px-4 py-2 text-sm text-red-300 bg-red-950/40">{gallery.error}</div>
  {/if}

  {#if !gallery.folder && !gallery.loading}
    <div class="flex-1 flex items-center justify-center text-white/40 text-sm">
      Open a folder or file, or drag and drop media here to get started.
    </div>
  {:else if gallery.view === "viewer"}
    <Viewer />
    <Filmstrip />
  {:else}
    <Grid />
  {/if}

  <SelectionTray />
</main>

{#if gallery.pendingDelete}
  <ConfirmDialog
    title={gallery.pendingDelete.length > 1
      ? `Move ${gallery.pendingDelete.length} files to trash?`
      : "Move this file to trash?"}
    message="You can restore it later, or press ⌘/Ctrl+Z to undo."
    confirmLabel="Move to Trash"
    onConfirm={(dontAskAgain) => gallery.confirmDelete(dontAskAgain)}
    onCancel={() => gallery.cancelDelete()}
  />
{/if}

{#if gallery.pendingClearCache}
  <ConfirmDialog
    title="Clear history & cache?"
    message="This forgets recently opened folders and permanently purges trashed files — undo will no longer work for past deletes."
    confirmLabel="Clear"
    showDontAskAgain={false}
    onConfirm={() => gallery.confirmClearCache()}
    onCancel={() => gallery.cancelClearCache()}
  />
{/if}

{#if gallery.showHelp}
  <HelpModal onClose={() => (gallery.showHelp = false)} />
{/if}

{#if gallery.showDuplicates}
  <DuplicatesModal onClose={() => (gallery.showDuplicates = false)} />
{/if}
