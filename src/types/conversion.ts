export type ImageFormat =
  | "png"
  | "jpg"
  | "gif"
  | "bmp"
  | "ico"
  | "tiff"
  | "webp"
  | "avif"
  | "svg"
  | "tga"
  | "qoi"
  | "hdr"
  | "ppm"
  | "exr"
  | "psd"
  | "heif"
  | "jxl"
  | "jp2";

export type AudioFormat = "wav" | "mp3" | "flac" | "ogg" | "aac" | "aiff" | "alac" | "opus" | "wma" | "ac3" | "dts";

export type VideoFormat = "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv" | "mpeg" | "ts" | "3gp" | "m4v";

export type DataFormat = "json" | "yaml" | "toml" | "xml" | "md" | "html";

export type ArchiveFormat = "zip" | "tar" | "targz" | "tarbz2" | "tarxz" | "7z" | "tarzst";

export type DocumentFormat = "pdf" | "png" | "jpg" | "txt" | "docx" | "odt" | "xlsx" | "ods" | "csv" | "pptx" | "odp";

export type OutputFormat = ImageFormat | AudioFormat | VideoFormat | DataFormat | ArchiveFormat | DocumentFormat;

export type FileCategory = "image" | "audio" | "video" | "document" | "textdoc" | "spreadsheet" | "presentation" | "data" | "archive";

const IMAGE_EXTENSIONS: Record<string, string> = {
  png: "png",
  jpg: "jpg",
  jpeg: "jpg",
  gif: "gif",
  bmp: "bmp",
  ico: "ico",
  tif: "tiff",
  tiff: "tiff",
  webp: "webp",
  avif: "avif",
  svg: "svg",
  tga: "tga",
  dds: "dds",
  qoi: "qoi",
  hdr: "hdr",
  ppm: "ppm",
  pgm: "ppm",
  pbm: "ppm",
  exr: "exr",
  psd: "psd",
  heif: "heif",
  heic: "heif",
  cr2: "cr2",
  nef: "nef",
  arw: "arw",
  dng: "dng",
  orf: "orf",
  rw2: "rw2",
  jxl: "jxl",
  jp2: "jp2",
  j2k: "jp2",
};

const AUDIO_EXTENSIONS: Record<string, string> = {
  wav: "wav",
  mp3: "mp3",
  flac: "flac",
  ogg: "ogg",
  aac: "aac",
  aiff: "aiff",
  aif: "aiff",
  m4a: "aac",
  alac: "alac",
  opus: "opus",
  wma: "wma",
  ac3: "ac3",
  dts: "dts",
};

const VIDEO_EXTENSIONS: Record<string, string> = {
  mp4: "mp4",
  avi: "avi",
  mkv: "mkv",
  mov: "mov",
  webm: "webm",
  flv: "flv",
  wmv: "wmv",
  mpeg: "mpeg",
  mpg: "mpeg",
  ts: "ts",
  "3gp": "3gp",
  m4v: "m4v",
  vob: "vob",
};

const DOCUMENT_EXTENSIONS: Record<string, string> = {
  pdf: "pdf",
};

const TEXTDOC_EXTENSIONS: Record<string, string> = {
  docx: "docx",
  odt: "odt",
  txt: "txt",
  rtf: "rtf",
  epub: "epub",
};

const SPREADSHEET_EXTENSIONS: Record<string, string> = {
  xlsx: "xlsx",
  xls: "xlsx",
  ods: "ods",
  csv: "csv",
};

const PRESENTATION_EXTENSIONS: Record<string, string> = {
  pptx: "pptx",
  odp: "odp",
};

const DATA_EXTENSIONS: Record<string, string> = {
  json: "json",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  xml: "xml",
  md: "md",
  html: "html",
  htm: "html",
};

const ARCHIVE_EXTENSIONS: Record<string, string> = {
  zip: "zip",
  tar: "tar",
  gz: "gz",
  tgz: "tgz",
  bz2: "bz2",
  xz: "xz",
  "7z": "7z",
  rar: "rar",
  zst: "zst",
};

export function getFileCategory(extension: string): FileCategory | null {
  const ext = extension.toLowerCase().replace(/^\./, "");
  if (ext in IMAGE_EXTENSIONS) return "image";
  if (ext in AUDIO_EXTENSIONS) return "audio";
  if (ext in VIDEO_EXTENSIONS) return "video";
  if (ext in DOCUMENT_EXTENSIONS) return "document";
  if (ext in TEXTDOC_EXTENSIONS) return "textdoc";
  if (ext in SPREADSHEET_EXTENSIONS) return "spreadsheet";
  if (ext in PRESENTATION_EXTENSIONS) return "presentation";
  if (ext in DATA_EXTENSIONS) return "data";
  if (ext in ARCHIVE_EXTENSIONS) return "archive";
  return null;
}

export function getOutputFormats(category: FileCategory): string[] {
  switch (category) {
    case "image":
      return ["png", "jpg", "webp", "bmp", "gif", "tiff", "avif", "ico", "tga", "qoi", "hdr", "ppm", "exr", "pdf"];
    case "audio":
      return ["wav", "flac", "aiff", "mp3", "ogg", "opus"];
    case "video":
      return ["mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "mpeg", "ts", "3gp", "m4v"];
    case "document":
      return ["png", "jpg"];
    case "textdoc":
      return ["txt", "pdf", "docx", "odt"];
    case "spreadsheet":
      return ["csv", "xlsx", "ods"];
    case "presentation":
      return ["pdf", "pptx", "odp"];
    case "data":
      return ["json", "yaml", "toml", "xml", "md", "html", "csv", "txt", "pdf"];
    case "archive":
      return ["zip", "tar", "targz", "tarbz2", "tarxz", "7z", "tarzst"];
  }
}

const FORMAT_TO_EXTENSION: Record<string, string> = {
  targz: "tar.gz",
  tarbz2: "tar.bz2",
  tarxz: "tar.xz",
  tarzst: "tar.zst",
};

export function getFileExtension(format: string): string {
  return FORMAT_TO_EXTENSION[format] ?? format;
}

export function getAllSupportedExtensions(): string[] {
  return [
    ...Object.keys(IMAGE_EXTENSIONS),
    ...Object.keys(AUDIO_EXTENSIONS),
    ...Object.keys(VIDEO_EXTENSIONS),
    ...Object.keys(DOCUMENT_EXTENSIONS),
    ...Object.keys(TEXTDOC_EXTENSIONS),
    ...Object.keys(SPREADSHEET_EXTENSIONS),
    ...Object.keys(PRESENTATION_EXTENSIONS),
    ...Object.keys(DATA_EXTENSIONS),
    ...Object.keys(ARCHIVE_EXTENSIONS),
  ];
}

export interface ConversionOptions {
  output_format: string;
  quality: number | null;
}

export interface ConversionJob {
  id: string;
  input_path: string;
  output_path: string;
  options: ConversionOptions;
}

export interface BatchConversionRequest {
  jobs: ConversionJob[];
}

export interface ProgressEvent {
  job_id: string;
  progress: number;
  status: JobStatus;
  error: string | null;
}

export type JobStatus = "pending" | "converting" | "done" | "error";

export interface BatchConversionResult {
  total: number;
  succeeded: number;
  failed: number;
  results: JobResult[];
}

export interface JobResult {
  job_id: string;
  success: boolean;
  output_path: string | null;
  error: string | null;
}

export interface FileInfo {
  path: string;
  name: string;
  extension: string;
  size: number;
  format: string | null;
}

export interface OutputFormatInfo {
  format: string;
  extension: string;
  label: string;
  supports_quality: boolean;
  category: string;
}

export interface ConversionFile extends FileInfo {
  id: string;
  progress: number;
  status: JobStatus;
  error: string | null;
  outputPath: string | null;
  category: FileCategory;
}

export const FORMAT_OPTIONS: Record<string, { label: string; color: string }> = {
  png: { label: "PNG", color: "bg-blue-600" },
  jpg: { label: "JPG", color: "bg-amber-600" },
  webp: { label: "WebP", color: "bg-green-600" },
  bmp: { label: "BMP", color: "bg-purple-600" },
  gif: { label: "GIF", color: "bg-pink-600" },
  tiff: { label: "TIFF", color: "bg-cyan-600" },
  avif: { label: "AVIF", color: "bg-red-600" },
  ico: { label: "ICO", color: "bg-indigo-600" },
  svg: { label: "SVG", color: "bg-orange-600" },
  tga: { label: "TGA", color: "bg-violet-600" },
  qoi: { label: "QOI", color: "bg-lime-600" },
  hdr: { label: "HDR", color: "bg-yellow-700" },
  ppm: { label: "PPM", color: "bg-stone-500" },
  exr: { label: "EXR", color: "bg-zinc-600" },
  psd: { label: "PSD", color: "bg-blue-800" },
  heif: { label: "HEIF", color: "bg-pink-700" },
  jxl: { label: "JXL", color: "bg-violet-700" },
  jp2: { label: "JP2", color: "bg-fuchsia-800" },
  pdf: { label: "PDF", color: "bg-rose-600" },

  wav: { label: "WAV", color: "bg-sky-600" },
  mp3: { label: "MP3", color: "bg-fuchsia-600" },
  flac: { label: "FLAC", color: "bg-teal-600" },
  ogg: { label: "OGG", color: "bg-yellow-600" },
  aac: { label: "AAC", color: "bg-emerald-600" },
  aiff: { label: "AIFF", color: "bg-slate-600" },
  alac: { label: "ALAC", color: "bg-emerald-800" },
  opus: { label: "OPUS", color: "bg-indigo-700" },
  wma: { label: "WMA", color: "bg-blue-900" },
  ac3: { label: "AC3", color: "bg-gray-700" },
  dts: { label: "DTS", color: "bg-gray-800" },

  mp4: { label: "MP4", color: "bg-red-700" },
  avi: { label: "AVI", color: "bg-amber-800" },
  mkv: { label: "MKV", color: "bg-cyan-700" },
  mov: { label: "MOV", color: "bg-gray-600" },
  webm: { label: "WebM", color: "bg-green-800" },
  flv: { label: "FLV", color: "bg-orange-800" },
  wmv: { label: "WMV", color: "bg-blue-900" },
  mpeg: { label: "MPEG", color: "bg-slate-700" },
  ts: { label: "TS", color: "bg-neutral-600" },
  "3gp": { label: "3GP", color: "bg-rose-700" },
  m4v: { label: "M4V", color: "bg-purple-700" },
  vob: { label: "VOB", color: "bg-stone-700" },

  json: { label: "JSON", color: "bg-yellow-600" },
  yaml: { label: "YAML", color: "bg-red-600" },
  toml: { label: "TOML", color: "bg-orange-600" },
  xml: { label: "XML", color: "bg-green-600" },
  md: { label: "MD", color: "bg-gray-500" },
  html: { label: "HTML", color: "bg-orange-500" },
  rtf: { label: "RTF", color: "bg-blue-500" },
  epub: { label: "EPUB", color: "bg-emerald-600" },

  txt: { label: "TXT", color: "bg-stone-600" },
  docx: { label: "DOCX", color: "bg-blue-700" },
  odt: { label: "ODT", color: "bg-sky-700" },
  xlsx: { label: "XLSX", color: "bg-green-700" },
  ods: { label: "ODS", color: "bg-emerald-700" },
  csv: { label: "CSV", color: "bg-teal-700" },
  pptx: { label: "PPTX", color: "bg-orange-700" },
  odp: { label: "ODP", color: "bg-amber-700" },

  zip: { label: "ZIP", color: "bg-amber-500" },
  tar: { label: "TAR", color: "bg-stone-700" },
  targz: { label: "TAR.GZ", color: "bg-stone-600" },
  tarbz2: { label: "TAR.BZ2", color: "bg-stone-500" },
  tarxz: { label: "TAR.XZ", color: "bg-stone-400" },
  "7z": { label: "7Z", color: "bg-cyan-800" },
  tarzst: { label: "TAR.ZST", color: "bg-teal-800" },
  rar: { label: "RAR", color: "bg-purple-800" },
};

export const FORMAT_DESCRIPTIONS: Record<string, string> = {
  png: "Lossless, transparency",
  jpg: "Lossy, small size",
  webp: "Modern, best compression",
  bmp: "Uncompressed bitmap",
  gif: "Animation support",
  tiff: "High quality, large",
  avif: "Next-gen, best quality",
  ico: "Windows icon (256x256)",
  tga: "Targa, legacy format",
  qoi: "Quite OK Image, fast",
  hdr: "Radiance HDR, high dynamic range",
  ppm: "Netpbm, scientific use",
  exr: "OpenEXR, HDR/VFX",
  psd: "Adobe Photoshop (input)",
  heif: "iPhone/Android photos (input)",
  jxl: "JPEG XL, next-gen",
  jp2: "JPEG 2000, medical/archive (input)",
  pdf: "Document format",

  wav: "Uncompressed audio",
  mp3: "Compressed, universal",
  flac: "Lossless audio",
  ogg: "Open source, compressed",
  aac: "Advanced audio codec",
  aiff: "Apple uncompressed audio",
  alac: "Apple lossless (input)",
  opus: "Best quality/size ratio",
  wma: "Windows Media Audio (input)",
  ac3: "Dolby Digital (input)",
  dts: "Home cinema (input)",

  mp4: "Universal video, H.264",
  avi: "Legacy video, large files",
  mkv: "Flexible container",
  mov: "Apple QuickTime",
  webm: "Web video, VP9/AV1",
  flv: "Flash video (legacy)",
  wmv: "Windows Media Video",
  mpeg: "Legacy broadcast",
  ts: "Transport Stream, TV",
  "3gp": "Mobile legacy",
  m4v: "Apple DRM-free video",
  vob: "DVD video (input)",

  json: "JavaScript Object Notation",
  yaml: "YAML data format",
  toml: "TOML config format",
  xml: "Extensible Markup Language",
  md: "Markdown text",
  html: "Web page format",
  rtf: "Rich Text Format (input)",
  epub: "E-book format (input)",

  txt: "Plain text, universal",
  docx: "Word document",
  odt: "OpenDocument text",
  xlsx: "Excel spreadsheet",
  ods: "OpenDocument spreadsheet",
  csv: "Comma-separated values",
  pptx: "PowerPoint presentation",
  odp: "OpenDocument presentation",

  zip: "Most common archive",
  tar: "Unix tape archive",
  targz: "Gzip compressed tar",
  tarbz2: "Bzip2 compressed tar",
  tarxz: "XZ compressed tar",
  "7z": "7-Zip, high compression",
  tarzst: "Zstandard compressed tar",
  rar: "RAR archive (input)",
};

export const LOSSY_FORMATS: string[] = ["jpg", "webp", "avif", "mp3", "ogg", "opus"];

export const CATEGORY_LABELS: Record<FileCategory, string> = {
  image: "Image",
  audio: "Audio",
  video: "Video",
  document: "Document",
  textdoc: "Text Doc",
  spreadsheet: "Spreadsheet",
  presentation: "Presentation",
  data: "Data",
  archive: "Archive",
};

export const CATEGORY_COLORS: Record<FileCategory, string> = {
  image: "bg-blue-500/15 text-blue-400",
  audio: "bg-purple-500/15 text-purple-400",
  video: "bg-red-500/15 text-red-400",
  document: "bg-rose-500/15 text-rose-400",
  textdoc: "bg-sky-500/15 text-sky-400",
  spreadsheet: "bg-green-500/15 text-green-400",
  presentation: "bg-orange-500/15 text-orange-400",
  data: "bg-yellow-500/15 text-yellow-400",
  archive: "bg-stone-500/15 text-stone-400",
};
