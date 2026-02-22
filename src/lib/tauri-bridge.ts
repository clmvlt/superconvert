import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  BatchConversionRequest,
  BatchConversionResult,
  FileInfo,
  OutputFormatInfo,
  ProgressEvent,
} from "../types/conversion";

export async function getFilesInfo(paths: string[]): Promise<FileInfo[]> {
  return invoke<FileInfo[]>("get_files_info", { paths });
}

export async function getOutputFormats(): Promise<OutputFormatInfo[]> {
  return invoke<OutputFormatInfo[]>("get_output_formats");
}

export async function convertFiles(
  request: BatchConversionRequest
): Promise<BatchConversionResult> {
  return invoke<BatchConversionResult>("convert_files", { request });
}

export async function scanDirectory(path: string): Promise<FileInfo[]> {
  return invoke<FileInfo[]>("scan_directory", { path });
}

export async function openPath(path: string): Promise<void> {
  await invoke("open_path", { path });
}

export async function onConversionProgress(
  callback: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<ProgressEvent>("conversion:progress", (event) => {
    callback(event.payload);
  });
}
