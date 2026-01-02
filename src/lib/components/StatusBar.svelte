<script lang="ts">
  import { editorStore } from "../stores/editor.svelte";

  const saveLabel = $derived(
    editorStore.isSaving
      ? "Saving..."
      : editorStore.isDirty
        ? "Unsaved"
        : "Saved"
  );
</script>

<footer class="status-bar">
  <div class="status-left">
    <span class="word-count">{editorStore.wordCount} words</span>
  </div>

  <div class="status-center">
    <span class="shortcut-hint">
      <kbd>Ctrl</kbd><kbd>/</kbd>
      <span class="hint-text">shortcuts</span>
    </span>
  </div>

  <div class="status-right">
    <span class="save-status" class:unsaved={editorStore.isDirty}>
      {saveLabel}
    </span>
  </div>
</footer>

<style>
  .status-bar {
    height: var(--statusbar-height);
    background: var(--surface-1);
    border-top: 1px solid var(--divider);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-md);
    font-size: var(--font-size-xs);
    color: var(--text-disabled);
    flex-shrink: 0;
    flex-wrap: nowrap;
    overflow: hidden;
    white-space: nowrap;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex-shrink: 0;
  }

  .status-center {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .shortcut-hint {
    display: flex;
    align-items: center;
    gap: 3px;
    opacity: 0.6;
    transition: opacity var(--transition-fast);
  }

  .shortcut-hint:hover {
    opacity: 1;
  }

  .shortcut-hint kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-0);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  .hint-text {
    margin-left: 4px;
    color: var(--text-disabled);
  }

  .save-status.unsaved {
    color: var(--warning);
  }

  .status-right {
    justify-content: flex-end;
  }
</style>
