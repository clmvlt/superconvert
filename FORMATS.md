# Convertor - Formats Supportés (Roadmap Complète)

## Images Raster

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| PNG | .png | ✅ | ✅ | 1 | `image` | Lossless, transparence alpha |
| JPEG | .jpg .jpeg | ✅ | ✅ | 1 | `image` | Lossy, qualité réglable |
| WebP | .webp | ✅ | ✅ | 1 | `image` | Lossy/lossless, moderne |
| BMP | .bmp | ✅ | ✅ | 1 | `image` | Non compressé |
| GIF | .gif | ✅ | ✅ | 1 | `image` | 256 couleurs, animation |
| TIFF | .tif .tiff | ✅ | ✅ | 1 | `image` | Pro/impression |
| AVIF | .avif | ✅ | ✅ | 2 | `image` | Successeur de JPEG, AV1 |
| ICO | .ico | ✅ | ✅ | 2 | `image` | Icônes Windows multi-taille |
| TGA | .tga | ✅ | ✅ | 2 | `image` | Jeux vidéo, 3D |
| DDS | .dds | ✅ | ❌ | 2 | `image` | Textures DirectX |
| QOI | .qoi | ✅ | ✅ | 2 | `image` | Lossless ultra-rapide |
| HEIF/HEIC | .heif .heic | ✅ | ❌ | 5 | `libheif-rs` | Photos iPhone/Android |
| Raw Photos | .cr2 .nef .arw .dng .orf .rw2 | ✅ | ❌ | 5 | `rawloader` + `imagepipe` | Appareils photo reflex |
| PSD | .psd | ✅ | ❌ | 5 | `psd` | Adobe Photoshop (aplati) |
| EXR | .exr | ✅ | ✅ | 5 | `exr` | HDR, cinéma, VFX |
| HDR | .hdr | ✅ | ✅ | 5 | `image` | Radiance HDR |
| PPM/PGM/PBM | .ppm .pgm .pbm | ✅ | ✅ | 5 | `image` | Netpbm, scientifique |
| JPEG XL | .jxl | ✅ | ✅ | 5 | `jxl-oxide` | Successeur JPEG, très performant |
| JPEG 2000 | .jp2 .j2k | ✅ | ❌ | 5 | `openjpeg-sys` | Médical, archives |

## Images Vectorielles

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| SVG | .svg .svgz | ✅ | ❌ | 2 | `resvg` + `usvg` | Vectoriel → raster uniquement |
| PDF (image) | .pdf | ❌ | ✅ | 2 | `printpdf` | Génération depuis image |

## Audio

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| WAV | .wav | ✅ | ✅ | 3 | `symphonia` / `hound` | PCM non compressé |
| MP3 | .mp3 | ✅ | ✅ | 3 | `symphonia` / `mp3lame-encoder` | Nécessite LAME pour encodage |
| FLAC | .flac | ✅ | ✅ | 3 | `symphonia` / `flac-bound` | Lossless |
| OGG Vorbis | .ogg | ✅ | ✅ | 3 | `symphonia` / `vorbis_encoder` | Libre, streaming |
| AAC | .aac .m4a | ✅ | ❌ | 3 | `symphonia` | Decode only (encode breveté) |
| AIFF | .aif .aiff | ✅ | ✅ | 3 | `symphonia` / `hound` | Apple PCM non compressé |
| ALAC | .alac .m4a | ✅ | ❌ | 3 | `symphonia` | Apple Lossless (decode) |
| Opus | .opus | ✅ | ✅ | 3 | `opus` / `audiopus` | Meilleur codec voix/musique |
| WMA | .wma | ✅ | ❌ | 4 | FFmpeg | Windows Media Audio |
| AC3 | .ac3 | ✅ | ❌ | 4 | FFmpeg | Dolby Digital, DVD |
| DTS | .dts | ✅ | ❌ | 4 | FFmpeg | Home cinéma |

## Vidéo

| Format | Extensions | Entrée | Sortie | Phase | Outil | Notes |
|--------|-----------|--------|--------|-------|-------|-------|
| MP4 | .mp4 | ✅ | ✅ | 4 | FFmpeg | H.264/H.265, le plus universel |
| AVI | .avi | ✅ | ✅ | 4 | FFmpeg | Legacy, gros fichiers |
| MKV | .mkv | ✅ | ✅ | 4 | FFmpeg | Conteneur flexible, sous-titres |
| MOV | .mov | ✅ | ✅ | 4 | FFmpeg | Apple QuickTime |
| WebM | .webm | ✅ | ✅ | 4 | FFmpeg | VP8/VP9/AV1, web |
| FLV | .flv | ✅ | ✅ | 4 | FFmpeg | Flash (legacy streaming) |
| WMV | .wmv | ✅ | ✅ | 4 | FFmpeg | Windows Media Video |
| MPEG/MPG | .mpeg .mpg | ✅ | ✅ | 4 | FFmpeg | Legacy broadcast |
| TS | .ts | ✅ | ✅ | 4 | FFmpeg | Transport Stream, TV |
| 3GP | .3gp | ✅ | ✅ | 4 | FFmpeg | Mobile legacy |
| M4V | .m4v | ✅ | ✅ | 4 | FFmpeg | Apple DRM-free video |
| VOB | .vob | ✅ | ❌ | 4 | FFmpeg | DVD |
| GIF animé | .gif | ✅ | ✅ | 4 | FFmpeg | Vidéo → GIF et inverse |

## Documents

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| PDF | .pdf | ✅ | ✅ | 4 | `lopdf` / `printpdf` | Lecture + génération |
| DOCX | .docx | ✅ | ❌ | 5 | `docx-rs` | Word → texte/PDF |
| XLSX | .xlsx | ✅ | ❌ | 5 | `calamine` | Excel → CSV |
| PPTX | .pptx | ✅ | ❌ | 5 | `pptx-rs` | PowerPoint → images/PDF |
| ODT | .odt | ✅ | ❌ | 5 | `odt-rs` ou ZIP + XML | LibreOffice Writer |
| ODS | .ods | ✅ | ❌ | 5 | `calamine` | LibreOffice Calc → CSV |
| ODP | .odp | ✅ | ❌ | 5 | ZIP + XML | LibreOffice Impress |
| CSV | .csv | ✅ | ✅ | 5 | `csv` | Données tabulaires |
| JSON | .json | ✅ | ✅ | 5 | `serde_json` | JSON ↔ CSV/YAML/TOML |
| YAML | .yaml .yml | ✅ | ✅ | 5 | `serde_yaml` | YAML ↔ JSON/TOML |
| TOML | .toml | ✅ | ✅ | 5 | `toml` | TOML ↔ JSON/YAML |
| XML | .xml | ✅ | ✅ | 5 | `quick-xml` | XML ↔ JSON |
| Markdown | .md | ✅ | ✅ | 5 | `pulldown-cmark` | MD → HTML/PDF |
| HTML | .html .htm | ✅ | ✅ | 5 | `pulldown-cmark` | HTML ↔ MD, HTML → PDF |
| TXT | .txt | ✅ | ✅ | 5 | natif | Encodage (UTF-8, Latin-1, etc.) |
| RTF | .rtf | ✅ | ❌ | 5 | `rtf-parser` | Rich Text → texte/PDF |
| EPUB | .epub | ✅ | ❌ | 5 | `epub-builder` | E-book → PDF/HTML |

## Archives (extraction/recompression)

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| ZIP | .zip | ✅ | ✅ | 5 | `zip` | Le plus courant |
| TAR | .tar | ✅ | ✅ | 5 | `tar` | Unix standard |
| GZ | .gz .tar.gz .tgz | ✅ | ✅ | 5 | `flate2` | Compression gzip |
| BZ2 | .bz2 .tar.bz2 | ✅ | ✅ | 5 | `bzip2` | Compression bzip2 |
| XZ | .xz .tar.xz | ✅ | ✅ | 5 | `xz2` | Compression LZMA |
| 7Z | .7z | ✅ | ✅ | 5 | `sevenz-rust` | 7-Zip |
| RAR | .rar | ✅ | ❌ | 5 | `unrar` | Extraction uniquement (propriétaire) |
| ZSTD | .zst .tar.zst | ✅ | ✅ | 5 | `zstd` | Compression moderne rapide |

## Polices

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| TTF | .ttf | ✅ | ✅ | 5 | `ttf-parser` + `woff` | TrueType |
| OTF | .otf | ✅ | ✅ | 5 | `ttf-parser` | OpenType |
| WOFF | .woff | ✅ | ✅ | 5 | `woff` | Web Open Font Format |
| WOFF2 | .woff2 | ✅ | ✅ | 5 | `woff2` | WOFF compressé (Brotli) |

## 3D / CAO (bonus)

| Format | Extensions | Entrée | Sortie | Phase | Crate Rust | Notes |
|--------|-----------|--------|--------|-------|------------|-------|
| STL | .stl | ✅ | ✅ | 5+ | `stl_io` | Impression 3D |
| OBJ | .obj | ✅ | ✅ | 5+ | `tobj` | Wavefront 3D |
| GLTF/GLB | .gltf .glb | ✅ | ✅ | 5+ | `gltf` | Standard web 3D |

## Matrice de Conversion Prioritaire (Phase 1-2)

```
        → PNG  JPG  WebP  BMP  GIF  TIFF  AVIF  ICO  PDF
PNG       -    ✅   ✅    ✅   ✅   ✅    ✅    ✅   ✅
JPG      ✅    -    ✅    ✅   ✅   ✅    ✅    ✅   ✅
WebP     ✅   ✅     -    ✅   ✅   ✅    ✅    ✅   ✅
BMP      ✅   ✅    ✅     -   ✅   ✅    ✅    ✅   ✅
GIF      ✅   ✅    ✅    ✅    -   ✅    ✅    ❌   ❌
TIFF     ✅   ✅    ✅    ✅   ✅    -    ✅    ✅   ✅
AVIF     ✅   ✅    ✅    ✅   ✅   ✅     -    ✅   ✅
ICO      ✅   ✅    ✅    ✅   ❌   ❌    ❌    -    ❌
SVG      ✅   ✅    ✅    ✅   ❌   ✅    ✅    ✅   ✅
TGA      ✅   ✅    ✅    ✅   ✅   ✅    ✅    ❌   ❌
QOI      ✅   ✅    ✅    ✅   ✅   ✅    ✅    ❌   ❌
DDS      ✅   ✅    ✅    ✅   ❌   ❌    ❌    ❌   ❌
```

## Résumé par Catégorie

| Catégorie | Formats en entrée | Formats en sortie | Phase |
|-----------|------------------|-------------------|-------|
| Images raster | 18 | 13 | 1-2-5 |
| Images vectorielles | 1 (SVG) | 0 (raster only) | 2 |
| Audio | 11 | 7 | 3-4 |
| Vidéo | 13 | 11 | 4 |
| Documents | 16 | 10 | 4-5 |
| Archives | 8 | 7 | 5 |
| Polices | 4 | 4 | 5 |
| 3D | 3 | 3 | 5+ |

**Total : ~74 formats en entrée, ~55 formats en sortie**
