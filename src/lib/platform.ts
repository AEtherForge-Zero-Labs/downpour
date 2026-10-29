import { invoke, isTauri, Channel } from '@tauri-apps/api/core';
export const native = isTauri();
export interface Options {
  url: string; mode: 'video' | 'audio'; format: string; quality: string;
  directory: string; playlist: boolean; subtitles: boolean; thumbnail: boolean;
  metadata: boolean; rateLimit: string; concurrentFragments: number;
}
export interface DownloadEvent { kind: 'log' | 'complete' | 'error' | 'cancelled'; text: string }
export interface Dependencies { ytDlp: string | null; ffmpeg: boolean; directory: string }
export interface Release { version: string; url: string }
export const dependencies = () => invoke<Dependencies>('dependencies');
export const latestRelease = () => invoke<Release>('latest_release');
export const preview = (options: Options) => invoke<string[]>('preview', { options });
export const chooseDirectory = () => invoke<string | null>('choose_directory');
export const cancel = () => invoke('cancel_download');
export const reveal = (directory: string) => invoke('open_directory', { directory });
export function download(options: Options, onEvent: (e: DownloadEvent) => void) {
  const events = new Channel<DownloadEvent>(); events.onmessage = onEvent;
  return invoke('start_download', { options, events });
}
