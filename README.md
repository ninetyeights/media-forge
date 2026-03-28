# Media Forge

A cross-platform desktop media processing tool built with Tauri 2 + Svelte 5 + Rust. Focused on batch image compression, format conversion, and automated file watching.

## Features

### Image Processing
- **Batch compression** with 7 presets (lossless, light, standard, max, target size, web-optimized)
- **Format conversion** between JPG, PNG, WebP, AVIF, GIF, BMP, TIFF
- **Target size compression** with binary search for optimal quality
- **Resize** with aspect ratio preservation
- **EXIF stripping**
- **File size filtering** (min/max range)
- **Folder tree view** with collapsible directory hierarchy
- **Batch selection** with per-file and per-folder checkboxes
- **Concurrent processing** (configurable 1-16 threads)
- High-performance encoding: turbojpeg (SIMD), libwebp, oxipng + imagequant

### File Watcher
- **Multi-folder monitoring** with independent per-folder settings
- **Per-folder controls**: media type, file extension filter, compression preset, output format, output directory, recursive toggle
- **Auto-processing** with configurable presets per folder
- **File stability detection** (waits for file write completion before processing)
- **Feedback loop prevention** (counter-based ignore with TTL expiry)
- **Processing logs** per folder with timestamps and detailed process info
- **Log persistence** across app restarts
- **Popover settings UI** for clean folder configuration

### System Tray
- Tray icon with real-time status text (watching count, stopped)
- Per-folder submenu with status and toggle controls
- Start/stop watching directly from tray

### Settings
- Concurrent processing count
- File size display unit (1024 / 1000)
- Default output directory
- System notifications toggle
- Auto-start on login
- All settings persisted automatically

### UI/UX
- Native macOS vibrancy / Windows Mica effects
- Dark theme with Catppuccin-inspired colors
- Custom title bar with window controls
- Drag-and-drop file/folder import
- System notifications for background processing

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2 |
| Frontend | Svelte 5 (Runes), TypeScript, Tailwind CSS 4 |
| Backend | Rust |
| Image Processing | image, turbojpeg, webp, oxipng, imagequant |
| File Watching | notify 7, notify-debouncer-mini |
| Persistence | tauri-plugin-store |

## Development

```bash
# Install dependencies
pnpm install

# Run dev server
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Build Targets

The GitHub Actions workflow builds for:
- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)

Create a tag to trigger a release build:
```bash
git tag v0.1.0
git push origin v0.1.0
```

## Roadmap

### Not Yet Implemented
- **Video processing** - UI exists, backend not implemented (H.264, H.265, VP9, AV1 codec settings)
- **Audio processing** - UI exists, backend not implemented (bitrate, sample rate, format conversion)
- **Watermark** - UI exists, backend not implemented
- **History** - Processing history page placeholder
- **Image preview** - Before/after comparison view

### Planned Improvements
- Batch file renaming with templates
- Per-file processing progress bar
- Drag-and-drop reorder in file list
- Output filename templates
- Close to tray option

## License

MIT
