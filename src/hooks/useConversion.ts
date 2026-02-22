import { useReducer, useCallback, useEffect, useRef, useMemo } from "react";
import type {
  ConversionFile,
  ProgressEvent,
  FileInfo,
  JobStatus,
  JobResult,
  FileCategory,
} from "@/types/conversion";
import { getFileCategory, getOutputFormats, LOSSY_FORMATS } from "@/types/conversion";
import {
  getFilesInfo,
  convertFiles,
  scanDirectory,
  openPath,
  onConversionProgress,
} from "@/lib/tauri-bridge";
import { open } from "@tauri-apps/plugin-dialog";
import { generateId } from "@/lib/utils";

interface ConversionState {
  files: ConversionFile[];
  outputFormat: string;
  quality: number;
  outputFolder: string;
  isConverting: boolean;
  conversionDone: boolean;
  globalProgress: number;
}

type Action =
  | { type: "ADD_FILES"; files: FileInfo[] }
  | { type: "REMOVE_FILE"; id: string }
  | { type: "CLEAR_FILES" }
  | { type: "SET_FORMAT"; format: string }
  | { type: "SET_QUALITY"; quality: number }
  | { type: "SET_OUTPUT_FOLDER"; folder: string }
  | { type: "START_CONVERSION" }
  | { type: "UPDATE_PROGRESS"; event: ProgressEvent }
  | { type: "CONVERSION_COMPLETE"; results: JobResult[] }
  | { type: "RESET_CONVERSION" };

function resolveCategory(ext: string): FileCategory {
  return getFileCategory(ext) ?? "image";
}

function reducer(state: ConversionState, action: Action): ConversionState {
  switch (action.type) {
    case "ADD_FILES": {
      const existingPaths = new Set(state.files.map((f) => f.path));
      const newFiles: ConversionFile[] = action.files
        .filter((f) => f.format !== null && !existingPaths.has(f.path))
        .map((f) => ({
          ...f,
          id: generateId(),
          progress: 0,
          status: "pending" as JobStatus,
          error: null,
          outputPath: null,
          category: resolveCategory(f.extension),
        }));

      const allFiles = [...state.files, ...newFiles];

      const categories = new Set(allFiles.map((f) => f.category));
      let { outputFormat } = state;
      if (categories.size === 1) {
        const cat = [...categories][0];
        const validFormats = getOutputFormats(cat);
        if (!validFormats.includes(outputFormat)) {
          outputFormat = validFormats[0];
        }
      }

      return { ...state, files: allFiles, outputFormat, conversionDone: false };
    }
    case "REMOVE_FILE": {
      const files = state.files.filter((f) => f.id !== action.id);

      const categories = new Set(files.map((f) => f.category));
      let { outputFormat } = state;
      if (files.length > 0 && categories.size === 1) {
        const cat = [...categories][0];
        const validFormats = getOutputFormats(cat);
        if (!validFormats.includes(outputFormat)) {
          outputFormat = validFormats[0];
        }
      }

      return { ...state, files, outputFormat };
    }
    case "CLEAR_FILES":
      return { ...state, files: [], globalProgress: 0, conversionDone: false, outputFormat: "png" };
    case "SET_FORMAT":
      return { ...state, outputFormat: action.format };
    case "SET_QUALITY":
      return { ...state, quality: action.quality };
    case "SET_OUTPUT_FOLDER":
      return { ...state, outputFolder: action.folder };
    case "START_CONVERSION":
      return {
        ...state,
        isConverting: true,
        conversionDone: false,
        globalProgress: 0,
        files: state.files.map((f) => ({
          ...f,
          progress: 0,
          status: "pending" as JobStatus,
          error: null,
          outputPath: null,
        })),
      };
    case "UPDATE_PROGRESS": {
      const { event } = action;
      const files = state.files.map((f) =>
        f.id === event.job_id
          ? {
              ...f,
              progress: event.progress,
              status: event.status,
              error: event.error,
            }
          : f
      );
      const totalProgress =
        files.length > 0
          ? files.reduce((sum, f) => sum + f.progress, 0) / files.length
          : 0;
      return { ...state, files, globalProgress: totalProgress };
    }
    case "CONVERSION_COMPLETE": {
      const files = state.files.map((f) => {
        const result = action.results.find((r) => r.job_id === f.id);
        if (result && result.success && result.output_path) {
          return { ...f, outputPath: result.output_path };
        }
        return f;
      });
      return { ...state, files, isConverting: false, conversionDone: true };
    }
    case "RESET_CONVERSION":
      return {
        ...state,
        conversionDone: false,
        globalProgress: 0,
        files: state.files.map((f) => ({
          ...f,
          progress: 0,
          status: "pending" as JobStatus,
          error: null,
          outputPath: null,
        })),
      };
    default:
      return state;
  }
}

const initialState: ConversionState = {
  files: [],
  outputFormat: "png",
  quality: 85,
  outputFolder: "",
  isConverting: false,
  conversionDone: false,
  globalProgress: 0,
};

export function useConversion() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, []);

  const detectedCategory = useMemo<FileCategory | null>(() => {
    if (state.files.length === 0) return null;
    const categories = new Set(state.files.map((f) => f.category));
    if (categories.size === 1) return [...categories][0];
    return null;
  }, [state.files]);

  const availableFormats = useMemo<string[]>(() => {
    if (detectedCategory) {
      return getOutputFormats(detectedCategory);
    }
    if (state.files.length === 0) {
      return getOutputFormats("image");
    }
    const categories = new Set(state.files.map((f) => f.category));
    const formats = new Set<string>();
    for (const cat of categories) {
      for (const fmt of getOutputFormats(cat)) {
        formats.add(fmt);
      }
    }
    return [...formats];
  }, [state.files, detectedCategory]);

  const addFiles = useCallback(async (paths: string[]) => {
    const filesInfo = await getFilesInfo(paths);
    dispatch({ type: "ADD_FILES", files: filesInfo });
  }, []);

  const addFolder = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;
    const filesInfo = await scanDirectory(selected);
    dispatch({ type: "ADD_FILES", files: filesInfo });
  }, []);

  const removeFile = useCallback((id: string) => {
    dispatch({ type: "REMOVE_FILE", id });
  }, []);

  const clearFiles = useCallback(() => {
    dispatch({ type: "CLEAR_FILES" });
  }, []);

  const setFormat = useCallback((format: string) => {
    dispatch({ type: "SET_FORMAT", format });
  }, []);

  const setQuality = useCallback((quality: number) => {
    dispatch({ type: "SET_QUALITY", quality });
  }, []);

  const setOutputFolder = useCallback((folder: string) => {
    dispatch({ type: "SET_OUTPUT_FOLDER", folder });
  }, []);

  const startConversion = useCallback(async () => {
    if (state.files.length === 0) return;

    dispatch({ type: "START_CONVERSION" });

    const unlisten = await onConversionProgress((event) => {
      dispatch({ type: "UPDATE_PROGRESS", event });
    });
    unlistenRef.current = unlisten;

    const jobs = state.files.map((file) => {
      const inputPath = file.path;
      const dir = state.outputFolder || inputPath.substring(0, inputPath.lastIndexOf("\\") !== -1 ? inputPath.lastIndexOf("\\") : inputPath.lastIndexOf("/"));
      const baseName = file.name.substring(0, file.name.lastIndexOf("."));
      const ext = state.outputFormat;

      let outputPath = `${dir}\\${baseName}.${ext}`;
      if (outputPath === inputPath) {
        outputPath = `${dir}\\${baseName}_converted.${ext}`;
      }

      return {
        id: file.id,
        input_path: inputPath,
        output_path: outputPath,
        options: {
          output_format: state.outputFormat,
          quality: LOSSY_FORMATS.includes(state.outputFormat)
            ? state.quality
            : null,
        },
      };
    });

    try {
      const result = await convertFiles({ jobs });
      dispatch({ type: "CONVERSION_COMPLETE", results: result.results });
    } catch {
      dispatch({ type: "CONVERSION_COMPLETE", results: [] });
    } finally {
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    }
  }, [state.files, state.outputFormat, state.quality, state.outputFolder]);

  const openFile = useCallback(async (path: string) => {
    await openPath(path);
  }, []);

  const openOutputFolder = useCallback(async () => {
    const doneFile = state.files.find((f) => f.outputPath);
    if (!doneFile?.outputPath) return;

    const lastSep = Math.max(
      doneFile.outputPath.lastIndexOf("\\"),
      doneFile.outputPath.lastIndexOf("/")
    );
    const folder = doneFile.outputPath.substring(0, lastSep);
    await openPath(folder);
  }, [state.files]);

  const resetConversion = useCallback(() => {
    dispatch({ type: "RESET_CONVERSION" });
  }, []);

  return {
    ...state,
    detectedCategory,
    availableFormats,
    addFiles,
    addFolder,
    removeFile,
    clearFiles,
    setFormat,
    setQuality,
    setOutputFolder,
    startConversion,
    openFile,
    openOutputFolder,
    resetConversion,
  };
}
