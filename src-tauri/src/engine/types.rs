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
}

impl AudioFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![Self::Wav]
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
}

impl DocumentFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Pdf => "pdf",
        }
    }

    pub fn output_formats() -> Vec<Self> {
        vec![Self::Png, Self::Jpg]
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
}

impl OutputFormat {
    pub fn from_extension(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wav" => Some(Self::Audio(AudioFormat::Wav)),
            "flac" => Some(Self::Audio(AudioFormat::Flac)),
            "pdf" => Some(Self::Document(DocumentFormat::Pdf)),
            other => ImageFormat::from_extension(other).map(Self::Image),
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Image(f) => f.extension(),
            Self::Audio(f) => f.extension(),
            Self::Document(f) => f.extension(),
        }
    }

    #[allow(dead_code)]
    pub fn supports_quality(&self) -> bool {
        match self {
            Self::Image(f) => f.supports_quality(),
            Self::Audio(f) => f.supports_quality(),
            Self::Document(f) => f.supports_quality(),
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
        | "svg" | "tga" | "dds" | "qoi" => Some("image"),
        "mp3" | "wav" | "flac" | "ogg" | "aac" | "aiff" | "aif" | "m4a" => Some("audio"),
        "pdf" => Some("document"),
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
        "pdf" => Some("pdf".to_string()),
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
