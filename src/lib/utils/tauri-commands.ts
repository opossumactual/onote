import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs
export interface FolderInfo {
  name: string;
  path: string;
  children: FolderInfo[];
}

export interface NoteMeta {
  id: string;
  path: string;
  title: string;
  preview: string;
  modified: string;
  word_count: number;
}

export interface NoteContent {
  path: string;
  content: string;
}

export interface SearchResult {
  path: string;
  title: string;
  matches: SearchMatch[];
}

export interface SearchMatch {
  line_number: number;
  line_content: string;
}

export interface AppSettings {
  notes_dir: string;
  model: string;
  font_size: number;
}

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export interface WhisperModel {
  id: string;
  name: string;
  size_mb: number;
  description: string;
}

export interface ModelStatus {
  id: string;
  downloaded: boolean;
  path: string | null;
  size_mb: number;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
}

export interface AudioSamples {
  samples: number[];
  sample_rate: number;
}

// Note commands
export async function listFolders(): Promise<FolderInfo[]> {
  return invoke<FolderInfo[]>("list_folders");
}

export async function listNotes(folder: string): Promise<NoteMeta[]> {
  return invoke<NoteMeta[]>("list_notes", { folder });
}

export async function readNote(path: string): Promise<NoteContent> {
  return invoke<NoteContent>("read_note", { path });
}

export async function saveNote(path: string, content: string): Promise<void> {
  return invoke("save_note", { path, content });
}

export async function createNote(folder: string, title?: string): Promise<string> {
  return invoke<string>("create_note", { folder, title });
}

export async function deleteNote(path: string): Promise<void> {
  return invoke("delete_note", { path });
}

export async function searchNotes(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_notes", { query });
}

export async function createFolder(name: string, parent?: string): Promise<string> {
  return invoke<string>("create_folder", { name, parent });
}

export async function deleteFolder(path: string): Promise<void> {
  return invoke("delete_folder", { path });
}

export async function renameFolder(oldPath: string, newName: string): Promise<string> {
  return invoke<string>("rename_folder", { oldPath, newName });
}

// Settings commands
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

// Audio device commands
export async function listAudioDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("list_audio_devices");
}

export async function getSelectedDevice(): Promise<string | null> {
  return invoke<string | null>("get_selected_device");
}

export async function setSelectedDevice(deviceId: string): Promise<void> {
  return invoke("set_selected_device", { deviceId });
}

// Recording commands
export async function startRecording(): Promise<void> {
  return invoke("start_recording");
}

export async function stopRecording(): Promise<AudioSamples> {
  return invoke<AudioSamples>("stop_recording");
}

export async function cancelRecording(): Promise<void> {
  return invoke("cancel_recording");
}

export async function isRecording(): Promise<boolean> {
  return invoke<boolean>("is_recording");
}

// Whisper model commands
export async function listWhisperModels(): Promise<WhisperModel[]> {
  return invoke<WhisperModel[]>("list_whisper_models");
}

export async function getModelStatus(modelId: string): Promise<ModelStatus> {
  return invoke<ModelStatus>("get_model_status", { modelId });
}

export async function getSelectedModel(): Promise<string> {
  return invoke<string>("get_selected_model");
}

export async function setSelectedModel(modelId: string): Promise<void> {
  return invoke("set_selected_model", { modelId });
}

export async function downloadModel(modelId: string): Promise<string> {
  return invoke<string>("download_model", { modelId });
}

export async function transcribe(samples: number[]): Promise<string> {
  return invoke<string>("transcribe", { samples });
}

export async function deleteModel(modelId: string): Promise<void> {
  return invoke("delete_model", { modelId });
}
