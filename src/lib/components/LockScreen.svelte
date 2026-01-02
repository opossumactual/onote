<script lang="ts">
  import { vaultStore } from '../stores/vault.svelte';

  let password = $state('');
  let loading = $state(false);

  async function handleUnlock() {
    if (!password) return;
    loading = true;
    try {
      await vaultStore.unlock(password);
      password = '';
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleUnlock();
    }
  }
</script>

<div class="lock-screen">
  <div class="lock-container">
    <h1>ghostnote</h1>

    <input
      type="password"
      bind:value={password}
      onkeydown={handleKeydown}
      placeholder="Enter password"
      disabled={loading}
      autofocus
    />

    <button onclick={handleUnlock} disabled={loading || !password}>
      {loading ? 'Unlocking...' : 'Unlock'}
    </button>

    {#if vaultStore.error}
      <p class="error">{vaultStore.error}</p>
    {/if}

    <button class="link" onclick={() => { /* show recovery */ }}>
      Forgot password?
    </button>
  </div>
</div>

<style>
  .lock-screen {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    z-index: 9999;
  }

  .lock-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
  }

  h1 {
    color: var(--text-primary);
    font-size: 2rem;
    margin-bottom: 1rem;
  }

  input {
    width: 300px;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 1rem;
  }

  button {
    width: 300px;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 1rem;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.link {
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.875rem;
    width: auto;
  }

  .error {
    color: #ff4444;
    font-size: 0.875rem;
  }
</style>
