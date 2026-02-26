use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::error::ConversionError;
use crate::traits::Converter;
use crate::types::{ArchiveFormat, ConversionOptions, OutputFormat};

pub struct ArchiveConverter;

impl ArchiveConverter {
    pub fn new() -> Self {
        Self
    }

    fn extract_to_dir(
        &self,
        input: &Path,
        temp_dir: &Path,
    ) -> Result<(), ConversionError> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "zip" => self.extract_zip(input, temp_dir),
            "tar" => self.extract_tar(input, temp_dir),
            "gz" | "tgz" => self.extract_tar_gz(input, temp_dir),
            "bz2" => self.extract_tar_bz2(input, temp_dir),
            "xz" => self.extract_tar_xz(input, temp_dir),
            "7z" => self.extract_7z(input, temp_dir),
            "zst" => self.extract_tar_zst(input, temp_dir),
            #[cfg(feature = "rar")]
            "rar" => self.extract_rar(input, temp_dir),
            _ => Err(ConversionError::UnsupportedInputFormat(ext)),
        }
    }

    fn extract_zip(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ConversionError::ReadError(format!("ZIP error: {}", e)))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| ConversionError::ReadError(format!("ZIP entry error: {}", e)))?;

            let name = entry.name().to_string();
            let out_path = dest.join(&name);

            if entry.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = File::create(&out_path)?;
                std::io::copy(&mut entry, &mut out_file)?;
            }
        }
        Ok(())
    }

    fn extract_tar(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(dest)
            .map_err(|e| ConversionError::ReadError(format!("TAR error: {}", e)))?;
        Ok(())
    }

    fn extract_tar_gz(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        archive.unpack(dest)
            .map_err(|e| ConversionError::ReadError(format!("TAR.GZ error: {}", e)))?;
        Ok(())
    }

    fn extract_tar_bz2(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let bz2 = bzip2::read::BzDecoder::new(file);
        let mut archive = tar::Archive::new(bz2);
        archive.unpack(dest)
            .map_err(|e| ConversionError::ReadError(format!("TAR.BZ2 error: {}", e)))?;
        Ok(())
    }

    fn extract_tar_xz(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let xz = xz2::read::XzDecoder::new(file);
        let mut archive = tar::Archive::new(xz);
        archive.unpack(dest)
            .map_err(|e| ConversionError::ReadError(format!("TAR.XZ error: {}", e)))?;
        Ok(())
    }

    fn extract_7z(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        sevenz_rust::decompress_file(input, dest)
            .map_err(|e| ConversionError::ReadError(format!("7Z error: {}", e)))?;
        Ok(())
    }

    fn extract_tar_zst(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        let file = File::open(input)?;
        let zst = zstd::stream::Decoder::new(file)
            .map_err(|e| ConversionError::ReadError(format!("ZSTD error: {}", e)))?;
        let mut archive = tar::Archive::new(zst);
        archive.unpack(dest)
            .map_err(|e| ConversionError::ReadError(format!("TAR.ZSTD error: {}", e)))?;
        Ok(())
    }

    #[cfg(feature = "rar")]
    fn extract_rar(&self, input: &Path, dest: &Path) -> Result<(), ConversionError> {
        unrar::Archive::new(input)
            .extract_to(dest.to_string_lossy().to_string())
            .map_err(|e| ConversionError::ReadError(format!("RAR error: {:?}", e)))?
            .process()
            .map_err(|e| ConversionError::ReadError(format!("RAR process error: {:?}", e)))?;
        Ok(())
    }

    fn compress_dir(
        &self,
        source_dir: &Path,
        output: &Path,
        format: &ArchiveFormat,
    ) -> Result<(), ConversionError> {
        match format {
            ArchiveFormat::Zip => self.compress_zip(source_dir, output),
            ArchiveFormat::Tar => self.compress_tar(source_dir, output),
            ArchiveFormat::TarGz => self.compress_tar_gz(source_dir, output),
            ArchiveFormat::TarBz2 => self.compress_tar_bz2(source_dir, output),
            ArchiveFormat::TarXz => self.compress_tar_xz(source_dir, output),
            ArchiveFormat::SevenZ => self.compress_7z(source_dir, output),
            ArchiveFormat::TarZst => self.compress_tar_zst(source_dir, output),
        }
    }

    fn compress_zip(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        self.add_dir_to_zip(&mut zip_writer, source_dir, source_dir, options)?;

        zip_writer
            .finish()
            .map_err(|e| ConversionError::WriteError(format!("ZIP finish error: {}", e)))?;
        Ok(())
    }

    fn add_dir_to_zip(
        &self,
        zip_writer: &mut zip::ZipWriter<File>,
        base: &Path,
        current: &Path,
        options: zip::write::SimpleFileOptions,
    ) -> Result<(), ConversionError> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            if path.is_dir() {
                zip_writer
                    .add_directory(&format!("{}/", relative), options)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                self.add_dir_to_zip(zip_writer, base, &path, options)?;
            } else {
                zip_writer
                    .start_file(&relative, options)
                    .map_err(|e| ConversionError::WriteError(e.to_string()))?;
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip_writer.write_all(&buf)?;
            }
        }
        Ok(())
    }

    fn compress_tar(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let mut builder = tar::Builder::new(file);
        builder
            .append_dir_all(".", source_dir)
            .map_err(|e| ConversionError::WriteError(format!("TAR error: {}", e)))?;
        builder
            .finish()
            .map_err(|e| ConversionError::WriteError(format!("TAR finish error: {}", e)))?;
        Ok(())
    }

    fn compress_tar_gz(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(gz);
        builder
            .append_dir_all(".", source_dir)
            .map_err(|e| ConversionError::WriteError(format!("TAR.GZ error: {}", e)))?;
        let gz = builder
            .into_inner()
            .map_err(|e| ConversionError::WriteError(format!("TAR.GZ inner error: {}", e)))?;
        gz.finish()
            .map_err(|e| ConversionError::WriteError(format!("GZ finish error: {}", e)))?;
        Ok(())
    }

    fn compress_tar_bz2(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let bz2 = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
        let mut builder = tar::Builder::new(bz2);
        builder
            .append_dir_all(".", source_dir)
            .map_err(|e| ConversionError::WriteError(format!("TAR.BZ2 error: {}", e)))?;
        let bz2 = builder
            .into_inner()
            .map_err(|e| ConversionError::WriteError(format!("TAR.BZ2 inner error: {}", e)))?;
        bz2.finish()
            .map_err(|e| ConversionError::WriteError(format!("BZ2 finish error: {}", e)))?;
        Ok(())
    }

    fn compress_tar_xz(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let xz = xz2::write::XzEncoder::new(file, 6);
        let mut builder = tar::Builder::new(xz);
        builder
            .append_dir_all(".", source_dir)
            .map_err(|e| ConversionError::WriteError(format!("TAR.XZ error: {}", e)))?;
        let xz = builder
            .into_inner()
            .map_err(|e| ConversionError::WriteError(format!("TAR.XZ inner error: {}", e)))?;
        xz.finish()
            .map_err(|e| ConversionError::WriteError(format!("XZ finish error: {}", e)))?;
        Ok(())
    }

    fn compress_7z(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        sevenz_rust::compress_to_path(source_dir, output)
            .map_err(|e| ConversionError::WriteError(format!("7Z compress error: {}", e)))?;
        Ok(())
    }

    fn compress_tar_zst(&self, source_dir: &Path, output: &Path) -> Result<(), ConversionError> {
        let file = File::create(output)?;
        let zst = zstd::stream::Encoder::new(file, 3)
            .map_err(|e| ConversionError::WriteError(format!("ZSTD error: {}", e)))?;
        let mut builder = tar::Builder::new(zst);
        builder
            .append_dir_all(".", source_dir)
            .map_err(|e| ConversionError::WriteError(format!("TAR.ZSTD error: {}", e)))?;
        let zst = builder
            .into_inner()
            .map_err(|e| ConversionError::WriteError(format!("TAR.ZSTD inner error: {}", e)))?;
        zst.finish()
            .map_err(|e| ConversionError::WriteError(format!("ZSTD finish error: {}", e)))?;
        Ok(())
    }
}

impl Converter for ArchiveConverter {
    fn supported_input_formats(&self) -> &[&str] {
        &["zip", "tar", "gz", "tgz", "bz2", "xz", "7z", "zst", "rar"]
    }

    fn supported_output_formats(&self) -> &[&str] {
        &["zip", "tar", "targz", "tarbz2", "tarxz", "7z", "tarzst"]
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError> {
        on_progress(0.0);

        let output_format = match &options.output_format {
            OutputFormat::Archive(fmt) => fmt.clone(),
            _ => {
                return Err(ConversionError::UnsupportedOutputFormat(
                    options.output_format.extension().to_string(),
                ));
            }
        };

        let temp_dir = tempfile::tempdir()
            .map_err(|e| ConversionError::ConversionFailed(format!("Temp dir error: {}", e)))?;

        on_progress(0.1);

        self.extract_to_dir(input, temp_dir.path())?;

        on_progress(0.5);

        let real_ext = output_format.extension();
        let final_output = if output.extension().and_then(|e| e.to_str()).unwrap_or("") != real_ext
        {
            let stem = output
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            output.with_file_name(format!("{}.{}", stem, real_ext))
        } else {
            output.to_path_buf()
        };

        self.compress_dir(temp_dir.path(), &final_output, &output_format)?;

        on_progress(1.0);
        Ok(())
    }
}
