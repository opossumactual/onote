<script lang="ts">
  import RecordButton from "./RecordButton.svelte";
  import { notesStore } from "../stores/notes.svelte";
  import { editorStore } from "../stores/editor.svelte";
  import { uiStore } from "../stores/ui.svelte";

  interface Props {
    onshowshortcuts?: () => void;
  }

  let { onshowshortcuts }: Props = $props();

  async function handleNewNote() {
    const path = await notesStore.addNote();
    if (path) {
      editorStore.loadNote(path);
    }
  }

  function handleSettings() {
    uiStore.openSettings();
  }
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <button
      class="toolbar-btn panel-toggle"
      class:active={uiStore.sidebarVisible}
      onclick={() => uiStore.toggleSidebar()}
      title="Toggle sidebar (Ctrl+B)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <path d="M9 3v18" />
      </svg>
    </button>

    <button
      class="toolbar-btn panel-toggle"
      class:active={uiStore.noteListVisible}
      onclick={() => uiStore.toggleNoteList()}
      title="Toggle note list (Ctrl+L)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
      </svg>
    </button>

    <div class="toolbar-divider"></div>

    <button class="toolbar-btn" onclick={handleNewNote} title="New note (Ctrl+N)">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
      <span>New</span>
    </button>

    <RecordButton />
  </div>

  <div class="toolbar-center">
    <span class="app-title">opnotes</span>
  </div>

  <div class="toolbar-right">
    <button
      class="toolbar-btn help-btn"
      onclick={onshowshortcuts}
      title="Keyboard shortcuts (Ctrl+/)"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3" />
        <path d="M12 17h.01" />
      </svg>
    </button>

    <button class="toolbar-btn" onclick={handleSettings} title="Settings">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" />
      </svg>
    </button>
  </div>
</header>

<style>
  .toolbar {
    height: var(--toolbar-height);
    background: var(--surface-1);
    border-bottom: 1px solid var(--divider);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    flex-shrink: 0;
    position: relative;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .toolbar-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
  }

  .app-title {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-weight: 500;
    user-select: none;
    letter-spacing: -0.01em;
  }

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: var(--border-subtle);
    margin: 0 var(--space-xs);
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-radius: 6px;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .toolbar-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .toolbar-btn span {
    font-size: var(--font-size-sm);
  }

  /* Panel toggle buttons */
  .panel-toggle {
    padding: var(--space-xs);
    position: relative;
  }

  .panel-toggle.active {
    color: var(--accent);
    background: rgba(92, 124, 250, 0.1);
  }

  .panel-toggle.active:hover {
    background: rgba(92, 124, 250, 0.15);
    color: var(--accent-hover);
  }

  /* Help button with subtle keyboard hint */
  .help-btn {
    position: relative;
  }

  .help-btn::after {
    content: "?";
    position: absolute;
    top: -2px;
    right: -2px;
    width: 14px;
    height: 14px;
    font-size: 9px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-3);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    color: var(--text-disabled);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .help-btn:hover::after {
    opacity: 1;
  }
</style>
