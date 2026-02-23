import { useReducer, useCallback, useEffect, useRef, useMemo } from "react";
import type {
  ConversionFile,
  ProgressEvent,
  FileInfo,
  JobStatus,
  JobResult,
  FileCategory,
} from "@/types/conversion";
import { getFileCategory, getOutputFormats, LOSSY_FORMATS, getFileExtension } from "@/types/conversion";
import {
  getFilesInfo,
  convertFiles,
  scanDirectory,
  openPath,
  onConversionProgress,
} from "@/lib/tauri-bridge";
import { open } from "@tauri-apps/plugin-dialog";
import { generateId } from "@/lib/utils";

const CATEGORY_ORDER: FileCategory[] = [
  "image", "audio", "video", "document", "textdoc",
  "spreadsheet", "presentation", "data", "archive",
];

interface ConversionState {
  files: ConversionFile[];
  outputFormats: Partial<Record<FileCategory, string>>;
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
  | { type: "SET_FORMAT"; category: FileCategory; format: string }
  | { type: "SET_QUALITY"; quality: number }
  | { type: "SET_OUTPUT_FOLDER"; folder: string }
  | { type: "START_CONVERSION" }
  | { type: "START_SINGLE_CONVERSION"; id: string }
  | { type: "UPDATE_PROGRESS"; event: ProgressEvent }
  | { type: "CONVERSION_COMPLETE"; results: JobResult[] }
  | { type: "SINGLE_CONVERSION_COMPLETE"; result: JobResult }
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
      const outputFormats = { ...state.outputFormats };

      const categoriesPresent = new Set(allFiles.map((f) => f.category));
      for (const cat of categoriesPresent) {
        if (!outputFormats[cat]) {
          outputFormats[cat] = getOutputFormats(cat)[0];
        }
      }

      return { ...state, files: allFiles, outputFormats, conversionDone: false };
    }
    case "REMOVE_FILE": {
      const files = state.files.filter((f) => f.id !== action.id);
      const outputFormats = { ...state.outputFormats };

      const remainingCategories = new Set(files.map((f) => f.category));
      for (const cat of Object.keys(outputFormats) as FileCategory[]) {
        if (!remainingCategories.has(cat)) {
          delete outputFormats[cat];
        }
      }

      return { ...state, files, outputFormats };
    }
    case "CLEAR_FILES":
      return { ...state, files: [], globalProgress: 0, conversionDone: false, outputFormats: {} };
    case "SET_FORMAT":
      return {
        ...state,
        outputFormats: { ...state.outputFormats, [action.category]: action.format },
      };
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
    case "START_SINGLE_CONVERSION":
      return {
        ...state,
        files: state.files.map((f) =>
          f.id === action.id
            ? { ...f, progress: 0, status: "converting" as JobStatus, error: null, outputPath: null }
            : f
        ),
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
    case "SINGLE_CONVERSION_COMPLETE": {
      const { result } = action;
      const files = state.files.map((f) => {
        if (f.id !== result.job_id) return f;
        if (result.success && result.output_path) {
          return { ...f, status: "done" as JobStatus, progress: 1, outputPath: result.output_path };
        }
        return { ...f, status: "error" as JobStatus, error: result.error ?? "Unknown error" };
      });
      return { ...state, files };
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
  outputFormats: {},
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

  const presentCategories = useMemo<FileCategory[]>(() => {
    if (state.files.length === 0) return [];
    const cats = new Set(state.files.map((f) => f.category));
    return CATEGORY_ORDER.filter((c) => cats.has(c));
  }, [state.files]);

  const categoryFormats = useMemo<Partial<Record<FileCategory, string[]>>>(() => {
    const result: Partial<Record<FileCategory, string[]>> = {};
    for (const cat of presentCategories) {
      result[cat] = getOutputFormats(cat);
    }
    return result;
  }, [presentCategories]);

  const hasLossyFormat = useMemo<boolean>(() => {
    return Object.values(state.outputFormats).some((fmt) =>
      fmt ? LOSSY_FORMATS.includes(fmt) : false
    );
  }, [state.outputFormats]);

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

  const setFormat = useCallback((category: FileCategory, format: string) => {
    dispatch({ type: "SET_FORMAT", category, format });
  }, []);

  const setQuality = useCallback((quality: number) => {
    dispatch({ type: "SET_QUALITY", quality });
  }, []);

  const setOutputFolder = useCallback((folder: string) => {
    dispatch({ type: "SET_OUTPUT_FOLDER", folder });
  }, []);

  const convertSingleFile = useCallback(async (fileId: string, format: string) => {
    const file = state.files.find((f) => f.id === fileId);
    if (!file) return;

    dispatch({ type: "START_SINGLE_CONVERSION", id: fileId });

    const unlisten = await onConversionProgress((event) => {
      dispatch({ type: "UPDATE_PROGRESS", event });
    });

    const inputPath = file.path;
    const dir = state.outputFolder || inputPath.substring(0, inputPath.lastIndexOf("\\") !== -1 ? inputPath.lastIndexOf("\\") : inputPath.lastIndexOf("/"));
    const baseName = file.name.substring(0, file.name.lastIndexOf("."));
    const ext = getFileExtension(format);

    let outputPath = `${dir}\\${baseName}.${ext}`;
    if (outputPath === inputPath) {
      outputPath = `${dir}\\${baseName}_converted.${ext}`;
    }

    const jobs = [{
      id: file.id,
      input_path: inputPath,
      output_path: outputPath,
      options: {
        output_format: format,
        quality: LOSSY_FORMATS.includes(format) ? state.quality : null,
      },
    }];

    try {
      const result = await convertFiles({ jobs });
      if (result.results.length > 0) {
        dispatch({ type: "SINGLE_CONVERSION_COMPLETE", result: result.results[0] });
      }
    } catch {
      dispatch({ type: "SINGLE_CONVERSION_COMPLETE", result: { job_id: fileId, success: false, output_path: null, error: "Conversion failed" } });
    } finally {
      unlisten();
    }
  }, [state.files, state.quality, state.outputFolder]);

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
      const fileOutputFormat = state.outputFormats[file.category] ?? getOutputFormats(file.category)[0];
      const ext = getFileExtension(fileOutputFormat);

      let outputPath = `${dir}\\${baseName}.${ext}`;
      if (outputPath === inputPath) {
        outputPath = `${dir}\\${baseName}_converted.${ext}`;
      }

      return {
        id: file.id,
        input_path: inputPath,
        output_path: outputPath,
        options: {
          output_format: fileOutputFormat,
          quality: LOSSY_FORMATS.includes(fileOutputFormat) ? state.quality : null,
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
  }, [state.files, state.outputFormats, state.quality, state.outputFolder]);

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
    files: state.files,
    outputFormats: state.outputFormats,
    quality: state.quality,
    outputFolder: state.outputFolder,
    isConverting: state.isConverting,
    conversionDone: state.conversionDone,
    globalProgress: state.globalProgress,
    presentCategories,
    categoryFormats,
    hasLossyFormat,
    convertSingleFile,
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
