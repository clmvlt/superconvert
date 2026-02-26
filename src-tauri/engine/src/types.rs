use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpg,
    Gif,
    Bmp,
    Ico,
    Tiff,
    Webp,
    Avif,
    Svg,
    Tga,
    Dds,
    Qoi,
    Hdr,
    Ppm,
    Exr,
    Psd,
    Heif,
    RawPhoto,
    Jxl,
    Jp2,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpg),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "ico" => Some(Self::Ico),
            "tif" | "tiff" => Some(Self::Tiff),
            "webp" => Some(Self::Webp),
            "avif" => Some(Self::Avif),
            "svg" => Some(Self::Svg),
            "tga" => Some(Self::Tga),
            "dds" => Some(Self::Dds),
            "qoi" => Some(Self::Qoi),
            "hdr" => Some(Self::Hdr),
            "ppm" | "pgm" | "pbm" => Some(Self::Ppm),
            "exr" => Some(Self::Exr),
            "psd" => Some(Self::Psd),
            "heif" | "heic" => Some(Self::Heif),
            "cr2" | "nef" | "arw" | "dng" | "orf" | "rw2" => Some(Self::RawPhoto),
            "jxl" => Some(Self::Jxl),
            "jp2" | "j2k" => Some(Self::Jp2),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Ico => "ico",
            Self::Tiff => "tiff",
            Self::Webp => "webp",
            Self::Avif => "avif",
            Self::Svg => "svg",
            Self::Tga => "tga",
            Self::Dds => "dds",
            Self::Qoi => "qoi",
            Self::Hdr => "hdr",
            Self::Ppm => "ppm",
            Self::Exr => "exr",
            Self::Psd => "psd",
            Self::Heif => "heif",
            Self::RawPhoto => "dng",
            Self::Jxl => "jxl",
            Self::Jp2 => "jp2",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![
            Self::Png,
            Self::Jpg,
            Self::Webp,
            Self::Bmp,
            Self::Gif,
            Self::Tiff,
            Self::Avif,
            Self::Ico,
            Self::Tga,
            Self::Qoi,
            Self::Hdr,
            Self::Ppm,
            Self::Exr,
        ]
    }

    pub fn supports_quality(&self) -> bool {
        matches!(self, Self::Jpg | Self::Webp | Self::Avif)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Flac,
    Mp3,
    Ogg,
    Aac,
    Aiff,
    Alac,
    Opus,
    Wma,
    Ac3,
    Dts,
}

impl AudioFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            "mp3" => Some(Self::Mp3),
            "ogg" => Some(Self::Ogg),
            "aac" | "m4a" => Some(Self::Aac),
            "aiff" | "aif" => Some(Self::Aiff),
            "alac" => Some(Self::Alac),
            "opus" => Some(Self::Opus),
            "wma" => Some(Self::Wma),
            "ac3" => Some(Self::Ac3),
            "dts" => Some(Self::Dts),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Aac => "aac",
            Self::Aiff => "aiff",
            Self::Alac => "alac",
            Self::Opus => "opus",
            Self::Wma => "wma",
            Self::Ac3 => "ac3",
            Self::Dts => "dts",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        #[allow(unused_mut)]
        let mut formats = vec![Self::Wav, Self::Flac, Self::Aiff];

        #[cfg(feature = "mp3-encode")]
        formats.push(Self::Mp3);

        #[cfg(feature = "vorbis-encode")]
        formats.push(Self::Ogg);

        #[cfg(feature = "opus-encode")]
        formats.push(Self::Opus);

        formats
    }

    pub fn supports_quality(&self) -> bool {
        matches!(self, Self::Mp3 | Self::Ogg | Self::Opus)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Mp4,
    Avi,
    Mkv,
    Mov,
    Webm,
    Flv,
    Wmv,
    Mpeg,
    Ts,
    #[serde(rename = "3gp")]
    ThreeGp,
    M4v,
    Vob,
}

impl VideoFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" => Some(Self::Mp4),
            "avi" => Some(Self::Avi),
            "mkv" => Some(Self::Mkv),
            "mov" => Some(Self::Mov),
            "webm" => Some(Self::Webm),
            "flv" => Some(Self::Flv),
            "wmv" => Some(Self::Wmv),
            "mpeg" | "mpg" => Some(Self::Mpeg),
            "ts" => Some(Self::Ts),
            "3gp" => Some(Self::ThreeGp),
            "m4v" => Some(Self::M4v),
            "vob" => Some(Self::Vob),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Mp4 => "mp4",
            Self::Avi => "avi",
            Self::Mkv => "mkv",
            Self::Mov => "mov",
            Self::Webm => "webm",
            Self::Flv => "flv",
            Self::Wmv => "wmv",
            Self::Mpeg => "mpeg",
            Self::Ts => "ts",
            Self::ThreeGp => "3gp",
            Self::M4v => "m4v",
            Self::Vob => "vob",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![
            Self::Mp4,
            Self::Avi,
            Self::Mkv,
            Self::Mov,
            Self::Webm,
            Self::Flv,
            Self::Wmv,
            Self::Mpeg,
            Self::Ts,
            Self::ThreeGp,
            Self::M4v,
        ]
    }

    pub fn supports_quality(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataFormat {
    Json,
    Yaml,
    Toml,
    Xml,
    Markdown,
    Html,
    Csv,
    Txt,
}

impl DataFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "xml" => Some(Self::Xml),
            "md" => Some(Self::Markdown),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Xml => "xml",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Csv => "csv",
            Self::Txt => "txt",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![
            Self::Json,
            Self::Yaml,
            Self::Toml,
            Self::Xml,
            Self::Markdown,
            Self::Html,
            Self::Csv,
            Self::Txt,
        ]
    }

    pub fn supports_quality(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    Zip,
    Tar,
    #[serde(rename = "targz")]
    TarGz,
    #[serde(rename = "tarbz2")]
    TarBz2,
    #[serde(rename = "tarxz")]
    TarXz,
    #[serde(rename = "7z")]
    SevenZ,
    #[serde(rename = "tarzst")]
    TarZst,
}

impl ArchiveFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            "gz" | "tgz" => Some(Self::TarGz),
            "bz2" => Some(Self::TarBz2),
            "xz" => Some(Self::TarXz),
            "7z" => Some(Self::SevenZ),
            "zst" => Some(Self::TarZst),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::TarBz2 => "tar.bz2",
            Self::TarXz => "tar.xz",
            Self::SevenZ => "7z",
            Self::TarZst => "tar.zst",
        }
    }

    pub fn format_token(&self) -> &str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "targz",
            Self::TarBz2 => "tarbz2",
            Self::TarXz => "tarxz",
            Self::SevenZ => "7z",
            Self::TarZst => "tarzst",
        }
    }

    #[allow(dead_code)]
    pub fn from_token(token: &str) -> Option<Self> {
        match token.to_lowercase().as_str() {
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            "targz" => Some(Self::TarGz),
            "tarbz2" => Some(Self::TarBz2),
            "tarxz" => Some(Self::TarXz),
            "7z" => Some(Self::SevenZ),
            "tarzst" => Some(Self::TarZst),
            _ => None,
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![
            Self::Zip,
            Self::Tar,
            Self::TarGz,
            Self::TarBz2,
            Self::TarXz,
            Self::SevenZ,
            Self::TarZst,
        ]
    }

    pub fn supports_quality(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    Png,
    Jpg,
    Pdf,
    Txt,
    Docx,
    Odt,
    Xlsx,
    Ods,
    Csv,
    Pptx,
    Odp,
}

impl DocumentFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "txt" => Some(Self::Txt),
            "docx" => Some(Self::Docx),
            "odt" => Some(Self::Odt),
            "xlsx" => Some(Self::Xlsx),
            "ods" => Some(Self::Ods),
            "csv" => Some(Self::Csv),
            "pptx" => Some(Self::Pptx),
            "odp" => Some(Self::Odp),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Pdf => "pdf",
            Self::Txt => "txt",
            Self::Docx => "docx",
            Self::Odt => "odt",
            Self::Xlsx => "xlsx",
            Self::Ods => "ods",
            Self::Csv => "csv",
            Self::Pptx => "pptx",
            Self::Odp => "odp",
        }
    }

    pub fn document_output_formats() -> Vec<Self> {
        vec![Self::Pdf, Self::Docx, Self::Odt, Self::Txt, Self::Png, Self::Jpg]
    }

    pub fn spreadsheet_output_formats() -> Vec<Self> {
        vec![Self::Csv, Self::Xlsx, Self::Ods]
    }

    pub fn presentation_output_formats() -> Vec<Self> {
        vec![Self::Pdf, Self::Pptx, Self::Odp]
    }

    pub fn supports_quality(&self) -> bool {
        matches!(self, Self::Jpg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Image(ImageFormat),
    Audio(AudioFormat),
    Document(DocumentFormat),
    Video(VideoFormat),
    Data(DataFormat),
    Archive(ArchiveFormat),
}

impl OutputFormat {
    pub fn from_extension(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wav" => Some(Self::Audio(AudioFormat::Wav)),
            "flac" => Some(Self::Audio(AudioFormat::Flac)),
            "mp3" => Some(Self::Audio(AudioFormat::Mp3)),
            "ogg" => Some(Self::Audio(AudioFormat::Ogg)),
            "aac" => Some(Self::Audio(AudioFormat::Aac)),
            "aiff" => Some(Self::Audio(AudioFormat::Aiff)),
            "opus" => Some(Self::Audio(AudioFormat::Opus)),

            "pdf" => Some(Self::Document(DocumentFormat::Pdf)),
            "txt" => Some(Self::Document(DocumentFormat::Txt)),
            "docx" => Some(Self::Document(DocumentFormat::Docx)),
            "odt" => Some(Self::Document(DocumentFormat::Odt)),
            "xlsx" => Some(Self::Document(DocumentFormat::Xlsx)),
            "ods" => Some(Self::Document(DocumentFormat::Ods)),
            "csv" => Some(Self::Document(DocumentFormat::Csv)),
            "pptx" => Some(Self::Document(DocumentFormat::Pptx)),
            "odp" => Some(Self::Document(DocumentFormat::Odp)),

            "mp4" => Some(Self::Video(VideoFormat::Mp4)),
            "avi" => Some(Self::Video(VideoFormat::Avi)),
            "mkv" => Some(Self::Video(VideoFormat::Mkv)),
            "mov" => Some(Self::Video(VideoFormat::Mov)),
            "webm" => Some(Self::Video(VideoFormat::Webm)),
            "flv" => Some(Self::Video(VideoFormat::Flv)),
            "wmv" => Some(Self::Video(VideoFormat::Wmv)),
            "mpeg" => Some(Self::Video(VideoFormat::Mpeg)),
            "ts" => Some(Self::Video(VideoFormat::Ts)),
            "3gp" => Some(Self::Video(VideoFormat::ThreeGp)),
            "m4v" => Some(Self::Video(VideoFormat::M4v)),

            "json" => Some(Self::Data(DataFormat::Json)),
            "yaml" => Some(Self::Data(DataFormat::Yaml)),
            "toml" => Some(Self::Data(DataFormat::Toml)),
            "xml" => Some(Self::Data(DataFormat::Xml)),
            "md" => Some(Self::Data(DataFormat::Markdown)),
            "html" => Some(Self::Data(DataFormat::Html)),

            "zip" => Some(Self::Archive(ArchiveFormat::Zip)),
            "tar" => Some(Self::Archive(ArchiveFormat::Tar)),
            "targz" => Some(Self::Archive(ArchiveFormat::TarGz)),
            "tarbz2" => Some(Self::Archive(ArchiveFormat::TarBz2)),
            "tarxz" => Some(Self::Archive(ArchiveFormat::TarXz)),
            "7z" => Some(Self::Archive(ArchiveFormat::SevenZ)),
            "tarzst" => Some(Self::Archive(ArchiveFormat::TarZst)),

            other => ImageFormat::from_extension(other).map(Self::Image),
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Image(f) => f.extension(),
            Self::Audio(f) => f.extension(),
            Self::Document(f) => f.extension(),
            Self::Video(f) => f.extension(),
            Self::Data(f) => f.extension(),
            Self::Archive(f) => f.format_token(),
        }
    }

    #[allow(dead_code)]
    pub fn supports_quality(&self) -> bool {
        match self {
            Self::Image(f) => f.supports_quality(),
            Self::Audio(f) => f.supports_quality(),
            Self::Document(f) => f.supports_quality(),
            Self::Video(f) => f.supports_quality(),
            Self::Data(f) => f.supports_quality(),
            Self::Archive(f) => f.supports_quality(),
        }
    }
}

impl Serialize for OutputFormat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.extension())
    }
}

impl<'de> Deserialize<'de> for OutputFormat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_extension(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown format: {}", s)))
    }
}

pub fn file_category(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "tif" | "tiff" | "webp" | "avif"
        | "svg" | "tga" | "dds" | "qoi" | "hdr" | "ppm" | "pgm" | "pbm" | "exr" | "psd"
        | "heif" | "heic" | "cr2" | "nef" | "arw" | "dng" | "orf" | "rw2" | "jxl" | "jp2"
        | "j2k" => Some("image"),

        "mp3" | "wav" | "flac" | "ogg" | "aac" | "aiff" | "aif" | "m4a" | "alac" | "opus"
        | "wma" | "ac3" | "dts" => Some("audio"),

        "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv" | "mpeg" | "mpg" | "ts"
        | "3gp" | "m4v" | "vob" => Some("video"),

        "pdf" | "docx" | "odt" | "txt" | "rtf" | "epub" => Some("document"),
        "xlsx" | "xls" | "ods" | "csv" => Some("spreadsheet"),
        "pptx" | "odp" => Some("presentation"),

        "json" | "yaml" | "yml" | "toml" | "xml" | "md" | "html" | "htm" => Some("data"),

        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" | "zst" => {
            Some("archive")
        }

        _ => None,
    }
}

pub fn resolve_format(ext: &str) -> Option<String> {
    if let Some(fmt) = ImageFormat::from_extension(ext) {
        return Some(fmt.extension().to_string());
    }
    match ext.to_lowercase().as_str() {
        "mp3" => Some("mp3".to_string()),
        "wav" => Some("wav".to_string()),
        "flac" => Some("flac".to_string()),
        "ogg" => Some("ogg".to_string()),
        "aac" | "m4a" => Some("aac".to_string()),
        "aiff" | "aif" => Some("aiff".to_string()),
        "alac" => Some("alac".to_string()),
        "opus" => Some("opus".to_string()),
        "wma" => Some("wma".to_string()),
        "ac3" => Some("ac3".to_string()),
        "dts" => Some("dts".to_string()),

        "mp4" => Some("mp4".to_string()),
        "avi" => Some("avi".to_string()),
        "mkv" => Some("mkv".to_string()),
        "mov" => Some("mov".to_string()),
        "webm" => Some("webm".to_string()),
        "flv" => Some("flv".to_string()),
        "wmv" => Some("wmv".to_string()),
        "mpeg" | "mpg" => Some("mpeg".to_string()),
        "ts" => Some("ts".to_string()),
        "3gp" => Some("3gp".to_string()),
        "m4v" => Some("m4v".to_string()),
        "vob" => Some("vob".to_string()),

        "pdf" => Some("pdf".to_string()),
        "docx" => Some("docx".to_string()),
        "odt" => Some("odt".to_string()),
        "txt" => Some("txt".to_string()),
        "rtf" => Some("rtf".to_string()),
        "epub" => Some("epub".to_string()),
        "xlsx" => Some("xlsx".to_string()),
        "xls" => Some("xls".to_string()),
        "ods" => Some("ods".to_string()),
        "csv" => Some("csv".to_string()),
        "pptx" => Some("pptx".to_string()),
        "odp" => Some("odp".to_string()),

        "json" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "xml" => Some("xml".to_string()),
        "md" => Some("md".to_string()),
        "html" | "htm" => Some("html".to_string()),

        "zip" => Some("zip".to_string()),
        "tar" => Some("tar".to_string()),
        "gz" | "tgz" => Some("gz".to_string()),
        "bz2" => Some("bz2".to_string()),
        "xz" => Some("xz".to_string()),
        "7z" => Some("7z".to_string()),
        "rar" => Some("rar".to_string()),
        "zst" => Some("zst".to_string()),

        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub output_format: OutputFormat,
    pub quality: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionJob {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub options: ConversionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionRequest {
    pub jobs: Vec<ConversionJob>,
    #[serde(default)]
    pub delete_originals: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub progress: f32,
    pub status: JobStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Converting,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConversionResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<JobResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormatInfo {
    pub format: String,
    pub extension: String,
    pub label: String,
    pub supports_quality: bool,
    pub category: String,
}
