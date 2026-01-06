// Custom confirm dialog state (replaces window.confirm which doesn't work on macOS)

let isOpen = $state(false);
let message = $state("");
let resolvePromise: ((value: boolean) => void) | null = null;

export function showConfirm(msg: string): Promise<boolean> {
  message = msg;
  isOpen = true;

  return new Promise((resolve) => {
    resolvePromise = resolve;
  });
}

export function handleConfirm() {
  isOpen = false;
  resolvePromise?.(true);
  resolvePromise = null;
}

export function handleCancel() {
  isOpen = false;
  resolvePromise?.(false);
  resolvePromise = null;
}

export const confirmStore = {
  get isOpen() { return isOpen; },
  get message() { return message; },
  handleConfirm,
  handleCancel,
};
