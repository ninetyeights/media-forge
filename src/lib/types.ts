export interface MediaFile {
  id: string;
  path: string;
  name: string;
  baseDir: string;
  originalSize: number;
  format: string;
  width?: number;
  height?: number;
  duration?: number;
  status: "pending" | "processing" | "done" | "skipped" | "error";
  progress?: number;
  compressedSize?: number;
  ratio?: number;
  outputPath?: string;
  error?: string;
  mediaType: "image" | "video" | "audio";
  /** Description of what processing was applied */
  processInfo?: string;
  /** Timestamp when processing completed (or failed/skipped) */
  processedAt?: number;
}

export interface ScannedFile {
  path: string;
  base_dir: string;
}

export interface ImageInfo {
  width: number;
  height: number;
  format: string;
  file_size: number;
}

export interface ProcessResult {
  output_path: string;
  original_size: number;
  compressed_size: number;
  ratio: number;
}
