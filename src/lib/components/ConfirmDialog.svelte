<script lang="ts">
  let { message, onConfirm, onCancel }: {
    message: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onCancel();
    } else if (event.key === "Enter") {
      onConfirm();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="overlay" onclick={onCancel} role="presentation">
  <div class="dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
    <p class="message">{message}</p>
    <div class="buttons">
      <button class="btn cancel" onclick={onCancel}>Cancel</button>
      <button class="btn confirm" onclick={onConfirm}>Delete</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--surface-1);
    border: 1px solid var(--border-default);
    padding: var(--space-lg);
    min-width: 300px;
    max-width: 400px;
  }

  .message {
    color: var(--text-primary);
    margin-bottom: var(--space-lg);
    font-size: var(--font-size-sm);
  }

  .buttons {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
  }

  .btn {
    padding: var(--space-sm) var(--space-md);
    font-size: var(--font-size-sm);
    border: 1px solid var(--border-default);
    transition: all var(--transition-fast);
  }

  .btn.cancel {
    background: var(--surface-2);
    color: var(--text-secondary);
  }

  .btn.cancel:hover {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  .btn.confirm {
    background: var(--error-dim);
    color: var(--error);
    border-color: var(--error);
  }

  .btn.confirm:hover {
    background: var(--error);
    color: var(--surface-0);
  }
</style>
