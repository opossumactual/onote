<script lang="ts">
  import { vaultStore } from '../stores/vault.svelte';

  let password = $state('');
  let loading = $state(false);

  // Recovery mode state
  let showRecovery = $state(false);
  let recoveryKeyInput = $state('');
  let newPassword = $state('');
  let confirmNewPassword = $state('');
  let recoveryError = $state<string | null>(null);
  let copiedNewKey = $state(false);

  async function handleUnlock() {
    if (!password) return;
    loading = true;
    vaultStore.clearError();
    try {
      await vaultStore.unlock(password);
      password = '';
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (showRecovery && !vaultStore.recoveryKey) {
        handleRecover();
      } else if (!showRecovery) {
        handleUnlock();
      }
    }
  }

  async function handleRecover() {
    recoveryError = null;

    if (!recoveryKeyInput.trim()) {
      recoveryError = 'Please enter your recovery key';
      return;
    }

    if (newPassword.length < 8) {
      recoveryError = 'Password must be at least 8 characters';
      return;
    }

    if (newPassword !== confirmNewPassword) {
      recoveryError = 'Passwords do not match';
      return;
    }

    loading = true;
    try {
      await vaultStore.recover(recoveryKeyInput.trim(), newPassword);
      // Clear input fields (new recovery key is now in vaultStore.recoveryKey)
      recoveryKeyInput = '';
      newPassword = '';
      confirmNewPassword = '';
    } catch (e) {
      recoveryError = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function copyNewRecoveryKey() {
    if (vaultStore.recoveryKey) {
      await navigator.clipboard.writeText(vaultStore.recoveryKey);
      copiedNewKey = true;
    }
  }

  async function finishRecovery() {
    vaultStore.clearRecoveryKey();
    await vaultStore.checkStatus();
    showRecovery = false;
    copiedNewKey = false;
  }

  function toggleRecovery() {
    showRecovery = !showRecovery;
    recoveryError = null;
    vaultStore.clearError();
    // Clear fields when toggling
    password = '';
    recoveryKeyInput = '';
    newPassword = '';
    confirmNewPassword = '';
    copiedNewKey = false;
  }
</script>

<div class="lock-screen">
  <div class="lock-container">
    <h1>ghostnote</h1>

    {#if showRecovery}
      {#if vaultStore.recoveryKey}
        <!-- Show new recovery key after successful reset -->
        <h2>Save Your New Recovery Key</h2>
        <p class="subtitle warning">Your old recovery key no longer works. Save this new one!</p>

        <div class="recovery-key">
          <code>{vaultStore.recoveryKey}</code>
          <button class="copy" onclick={copyNewRecoveryKey}>
            {copiedNewKey ? 'Copied!' : 'Copy'}
          </button>
        </div>

        <button onclick={finishRecovery} disabled={!copiedNewKey}>
          I've Saved It
        </button>
      {:else}
        <p class="subtitle">Reset your password using your recovery key</p>

        <input
          type="text"
          bind:value={recoveryKeyInput}
          onkeydown={handleKeydown}
          placeholder="Recovery key (XXXX-XXXX-XXXX-XXXX-XXXX-XXXX)"
          disabled={loading}
          autofocus
        />

        <input
          type="password"
          bind:value={newPassword}
          onkeydown={handleKeydown}
          placeholder="New password (min 8 characters)"
          disabled={loading}
        />

        <input
          type="password"
          bind:value={confirmNewPassword}
          onkeydown={handleKeydown}
          placeholder="Confirm new password"
          disabled={loading}
        />

        {#if recoveryError}
          <p class="error">{recoveryError}</p>
        {/if}

        <button onclick={handleRecover} disabled={loading || !recoveryKeyInput || !newPassword || !confirmNewPassword}>
          {loading ? 'Resetting...' : 'Reset Password'}
        </button>

        <button class="link" onclick={toggleRecovery}>
          Back to login
        </button>
      {/if}
    {:else}
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

      <button class="link" onclick={toggleRecovery}>
        Forgot password?
      </button>
    {/if}
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

  h2 {
    color: var(--text-primary);
    font-size: 1.25rem;
    margin: 0;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin: 0;
    text-align: center;
  }

  .subtitle.warning {
    color: #ffaa44;
  }

  .recovery-key {
    width: 300px;
    padding: 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .recovery-key code {
    font-family: monospace;
    font-size: 0.9rem;
    color: var(--accent);
    word-break: break-all;
    line-height: 1.5;
    text-align: center;
  }

  .recovery-key .copy {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 0.875rem;
    padding: 0.5rem;
    width: 100%;
  }
</style>
