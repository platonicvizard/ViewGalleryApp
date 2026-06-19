<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { gallery } from "$lib/gallery.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import Viewer from "$lib/Viewer.svelte";
  import Filmstrip from "$lib/Filmstrip.svelte";
  import Grid from "$lib/Grid.svelte";

  function onKeydown(e: KeyboardEvent) {
    if (gallery.view !== "viewer") return;
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT") return;

    if (e.key === "ArrowRight") {
      e.preventDefault();
      gallery.next();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      gallery.prev();
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
      if (initial) await gallery.openPath(initial);

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
</main>
