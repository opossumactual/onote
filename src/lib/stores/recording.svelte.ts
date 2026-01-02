import * as commands from "../utils/tauri-commands";

// Recording State
type RecordingStatus = "idle" | "recording" | "processing";

let status = $state<RecordingStatus>("idle");
let duration = $state(0);
let error = $state<string | null>(null);

let intervalId: ReturnType<typeof setInterval>;

// Actions
async function startRecording() {
  error = null;

  try {
    await commands.startRecording();
    status = "recording";
    duration = 0;

    // Start duration counter
    intervalId = setInterval(() => {
      duration++;
    }, 1000);

    console.log("Recording started");
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    console.error("Failed to start recording:", error);
    status = "idle";
  }
}

async function stopRecording(): Promise<string | null> {
  clearInterval(intervalId);
  error = null;

  try {
    status = "processing";
    console.log("Stopping recording...");

    // Stop recording and get the audio file path
    const audioPath = await commands.stopRecording();
    console.log("Audio saved to:", audioPath);

    // Check if the model is downloaded
    const modelId = await commands.getSelectedModel();
    const modelStatus = await commands.getModelStatus(modelId);

    if (!modelStatus.downloaded) {
      error = `Model '${modelId}' not downloaded. Please download it in Settings first.`;
      status = "idle";
      duration = 0;
      return null;
    }

    // Transcribe the audio
    console.log("Transcribing with model:", modelId);
    const transcription = await commands.transcribe(audioPath);
    console.log("Transcription:", transcription);

    status = "idle";
    duration = 0;

    return transcription;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    console.error("Failed to process recording:", error);
    status = "idle";
    duration = 0;
    return null;
  }
}

async function cancelRecording() {
  clearInterval(intervalId);
  error = null;

  try {
    await commands.cancelRecording();
  } catch (e) {
    console.error("Failed to cancel recording:", e);
  }

  status = "idle";
  duration = 0;
}

// Format duration as MM:SS
function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

// Export reactive getters and actions
export const recordingStore = {
  get status() {
    return status;
  },
  get duration() {
    return duration;
  },
  get formattedDuration() {
    return formatDuration(duration);
  },
  get isRecording() {
    return status === "recording";
  },
  get isProcessing() {
    return status === "processing";
  },
  get error() {
    return error;
  },
  startRecording,
  stopRecording,
  cancelRecording,
  clearError() {
    error = null;
  },
};
