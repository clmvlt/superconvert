# Convertor

A fast, cross-platform file converter with a modern UI. Convert images, audio, video, and documents locally — no cloud, no upload, no subscription.

Built with **Tauri 2** (Rust backend) and **React 19** (TypeScript frontend).

## Features

- **Drag & drop** files directly into the app
- **Batch conversion** — convert multiple files at once
- **Parallel processing** — uses all CPU cores via Rayon
- **Real-time progress** tracking per file
- **OS context menu** integration (right-click → "Convert to...")
- **Cross-platform** — Windows, Linux, macOS
- **100% local** — your files never leave your machine

## Supported Formats

### Images

| Format | Input | Output |
|--------|:-----:|:------:|
| PNG    | ✅    | ✅     |
| JPEG   | ✅    | ✅     |
| WebP   | ✅    | ✅     |
| BMP    | ✅    | ✅     |
| GIF    | ✅    | ✅     |
| TIFF   | ✅    | ✅     |
| AVIF   | ✅    | ✅     |
| ICO    | ✅    | ✅     |
| SVG    | ✅    | —      |

### Audio

| Format | Input | Output |
|--------|:-----:|:------:|
| MP3    | ✅    | 🔜     |
| WAV    | ✅    | ✅     |
| FLAC   | ✅    | 🔜     |
| OGG    | ✅    | 🔜     |
| AAC    | ✅    | —      |

### Video (requires FFmpeg)

| Format | Input | Output |
|--------|:-----:|:------:|
| MP4    | ✅    | ✅     |
| AVI    | ✅    | ✅     |
| MKV    | ✅    | ✅     |
| MOV    | ✅    | ✅     |
| WebM   | ✅    | ✅     |

### Documents

| Conversion        | Status |
|-------------------|:------:|
| Image → PDF       | ✅     |
| PDF → Images      | ✅     |

> 🔜 = planned for a future release

## Prerequisites

### All platforms

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 18+
- [FFmpeg](https://ffmpeg.org/) (optional, for video conversion)

### Windows

- **Visual Studio Build Tools 2022** with "Desktop development with C++" workload
  - MSVC v143 build tools
  - Windows 11 SDK
- WebView2 (pre-installed on Windows 10/11)

### Linux

```bash
sudo apt install build-essential libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

See [Tauri prerequisites for Linux](https://v2.tauri.app/start/prerequisites/#linux).

### macOS

```bash
xcode-select --install
```

See [Tauri prerequisites for macOS](https://v2.tauri.app/start/prerequisites/#macos).

## Getting Started

```bash
git clone https://github.com/clmvlt/convertor.git
cd Convertor
npm install
```

## Development

Run the full Tauri app with hot reload:

```bash
npm run tauri dev
```

Run the frontend only (no Rust backend):

```bash
npm run dev
```

Lint the frontend:

```bash
npm run lint
```

Check Rust compilation:

```bash
cd src-tauri && cargo check
```

Run Rust tests:

```bash
cd src-tauri && cargo test
```

## Build

Build a production release for your current platform:

```bash
npm run tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`.

| Platform | Output                          |
|----------|---------------------------------|
| Windows  | `.msi` and `.exe` (NSIS)        |
| Linux    | `.deb`, `.rpm`, `.AppImage`     |
| macOS    | `.dmg`, `.app`                  |

## Tech Stack

| Layer    | Technology                         |
|----------|------------------------------------|
| Backend  | Rust + Tauri 2                     |
| Frontend | React 19 + TypeScript + Vite 7     |
| Styling  | Tailwind CSS 4                     |
| Images   | `image` crate + `resvg` (SVG)     |
| Audio    | `symphonia` (decode) + `hound` (WAV) |
| PDF      | `lopdf` + `printpdf`              |
| Video    | `ffmpeg-next` (planned)            |
| Async    | Tokio                              |
| Parallel | Rayon                              |

## Project Structure

```
Convertor/
├── src/                    # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/         # UI components
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utilities & Tauri bridge
│   └── types/              # Shared TypeScript types
│
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri setup
│   │   ├── commands/       # IPC command handlers
│   │   ├── engine/         # Conversion engines
│   │   ├── platform/       # OS-specific code (context menu)
│   │   ├── orchestrator.rs # Parallel conversion manager
│   │   └── progress.rs     # Progress tracking
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json
└── vite.config.ts
```

## License

MIT
