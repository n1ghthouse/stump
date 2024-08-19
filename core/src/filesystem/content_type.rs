use std::path::Path;

use serde::Serialize;

use crate::CoreError;

use super::image::ImageFormat;

/// [`ContentType`] is an enum that represents the HTTP content type. This is a smaller
/// subset of the full list of content types, mostly focusing on types supported by Stump.
#[allow(non_camel_case_types)]
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentType {
	XHTML,
	XML,
	HTML,
	PDF,
	EPUB_ZIP,
	ZIP,
	COMIC_ZIP,
	RAR,
	COMIC_RAR,
	PNG,
	JPEG,
	WEBP,
	AVIF,
	GIF,
	TXT,
	#[default]
	UNKNOWN,
}

fn temporary_content_workarounds(extension: &str) -> ContentType {
	if extension == "opf" || extension == "ncx" {
		return ContentType::XML;
	}

	ContentType::UNKNOWN
}

fn infer_mime_from_bytes(bytes: &[u8]) -> Option<String> {
	infer::get(bytes).map(|infer_type| infer_type.mime_type().to_string())
}

fn infer_mime(path: &Path) -> Option<String> {
	match infer::get_from_path(path) {
		Ok(result) => {
			tracing::trace!(?path, ?result, "inferred mime");
			result.map(|infer_type| infer_type.mime_type().to_string())
		},
		Err(e) => {
			tracing::trace!(error = ?e, ?path, "infer failed");
			None
		},
	}
}

impl ContentType {
	/// Infer the MIME type of a file extension.
	///
	/// ### Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::from_extension("png");
	/// assert_eq!(content_type, ContentType::PNG);
	/// ```
	pub fn from_extension(extension: &str) -> ContentType {
		match extension.to_lowercase().as_str() {
			"xhtml" => ContentType::XHTML,
			"xml" => ContentType::XML,
			"html" => ContentType::HTML,
			"pdf" => ContentType::PDF,
			"epub" => ContentType::EPUB_ZIP,
			"zip" => ContentType::ZIP,
			"cbz" => ContentType::COMIC_ZIP,
			"rar" => ContentType::RAR,
			"cbr" => ContentType::COMIC_RAR,
			"png" => ContentType::PNG,
			"jpg" => ContentType::JPEG,
			"jpeg" => ContentType::JPEG,
			"webp" => ContentType::WEBP,
			"avif" => ContentType::AVIF,
			"gif" => ContentType::GIF,
			"txt" => ContentType::TXT,
			_ => temporary_content_workarounds(extension),
		}
	}

	/// Infer the MIME type of a file using the [infer] crate. If the MIME type cannot be inferred,
	/// then the file extension is used to determine the content type.
	///
	/// ### Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::from_file("test.png");
	/// assert_eq!(content_type, ContentType::PNG);
	/// ```
	pub fn from_file(file_path: &str) -> ContentType {
		let path = Path::new(file_path);
		ContentType::from_path(path)
	}

	/// Infer the MIME type of a [Vec] of bytes using the [infer] crate. If the MIME type cannot be
	/// inferred, then the content type is set to [ContentType::UNKNOWN].
	///
	/// ### Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let buf = [0xFF, 0xD8, 0xFF, 0xAA];
	/// let content_type = ContentType::from_bytes(&buf);
	/// assert_eq!(content_type, ContentType::JPEG);
	/// ```
	pub fn from_bytes(bytes: &[u8]) -> ContentType {
		infer_mime_from_bytes(bytes)
			.map(|mime| ContentType::from(mime.as_str()))
			.unwrap_or_default()
	}

	/// Infer the MIME type of a [Vec] of bytes using the [infer] crate. If the MIME type cannot be
	/// inferred, then the extension is used to determine the content type.
	///
	/// ### Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// // This is NOT a valid PNG buff
	/// let buf = [0xFF, 0xD8, 0xBB, 0xBB];
	/// let content_type = ContentType::from_bytes_with_fallback(&buf, "png");
	/// assert_eq!(content_type, ContentType::PNG);
	/// ```
	pub fn from_bytes_with_fallback(bytes: &[u8], extension: &str) -> ContentType {
		infer_mime_from_bytes(bytes)
			.map(|mime| ContentType::from(mime.as_str()))
			.unwrap_or_else(|| {
				// NOTE: I am logging at warn level because inference from bytes is a little more
				// accurate, so if it fails it may be indicative of a problem.
				tracing::warn!(
					?bytes,
					?extension,
					"failed to infer content type, falling back to extension"
				);
				ContentType::from_extension(extension)
			})
	}

	/// Infer the MIME type of a [Path] using the [infer] crate. If the MIME type cannot be inferred,
	/// then the extension of the path is used to determine the content type.
	///
	/// ### Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	/// use std::path::Path;
	///
	/// let path = Path::new("test.png");
	/// let content_type = ContentType::from_path(path);
	/// assert_eq!(content_type, ContentType::PNG);
	/// ```
	pub fn from_path(path: &Path) -> ContentType {
		infer_mime(path)
			.map(|mime| ContentType::from(mime.as_str()))
			.unwrap_or_else(|| {
				ContentType::from_extension(
					path.extension()
						.unwrap_or_default()
						.to_str()
						.unwrap_or_default(),
				)
			})
	}

	/// Returns the string representation of the MIME type.
	pub fn mime_type(&self) -> String {
		self.to_string()
	}

	/// Returns true if the content type is an image.
	///
	/// ## Example
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::PNG;
	/// assert!(content_type.is_image());
	///
	/// let content_type = ContentType::XHTML;
	/// assert!(!content_type.is_image());
	/// ```
	pub fn is_image(&self) -> bool {
		self.to_string().starts_with("image")
	}

	/// Returns true if the content type is in accordance with the OPDS 1.2 specification.
	/// This includes PNG, JPEG, and GIF images.
	///
	/// ## Example
	///
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::PNG;
	/// assert!(content_type.is_opds_legacy_image());
	/// ```
	pub fn is_opds_legacy_image(&self) -> bool {
		self == &ContentType::PNG
			|| self == &ContentType::JPEG
			|| self == &ContentType::GIF
	}

	/// Returns true if the content type is a ZIP archive.
	///
	/// ## Example
	///
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::ZIP;
	/// assert!(content_type.is_zip());
	/// ```
	pub fn is_zip(&self) -> bool {
		self == &ContentType::ZIP || self == &ContentType::COMIC_ZIP
	}

	/// Returns true if the content type is a RAR archive.
	///
	/// ## Example
	///
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::RAR;
	/// assert!(content_type.is_rar());
	/// ```
	pub fn is_rar(&self) -> bool {
		self == &ContentType::RAR || self == &ContentType::COMIC_RAR
	}

	/// Returns true if the content type is an EPUB archive.
	///
	/// ## Example
	///
	/// ```rust
	/// use stump_core::filesystem::ContentType;
	///
	/// let content_type = ContentType::EPUB_ZIP;
	/// assert!(content_type.is_epub());
	/// ```
	pub fn is_epub(&self) -> bool {
		self == &ContentType::EPUB_ZIP
	}

	/// Returns the file extension of the content type. If the content type is unknown, then an
	/// empty string is returned.
	pub fn extension(&self) -> &str {
		match self {
			ContentType::XHTML => "xhtml",
			ContentType::XML => "xml",
			ContentType::HTML => "html",
			ContentType::PDF => "pdf",
			ContentType::EPUB_ZIP => "epub",
			ContentType::ZIP => "zip",
			ContentType::COMIC_ZIP => "cbz",
			ContentType::RAR => "rar",
			ContentType::COMIC_RAR => "cbr",
			ContentType::PNG => "png",
			ContentType::JPEG => "jpg",
			ContentType::WEBP => "webp",
			ContentType::AVIF => "avif",
			ContentType::GIF => "gif",
			ContentType::TXT => "txt",
			ContentType::UNKNOWN => "",
		}
	}
}

impl From<&str> for ContentType {
	/// Returns the content type from the string.
	///
	/// NOTE: It is assumed that the string is a valid representation of a content type.
	/// **Do not** use this method to parse a file path or extension.
	fn from(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"application/xhtml+xml" => ContentType::XHTML,
			"application/xml" => ContentType::XML,
			"text/html" => ContentType::HTML,
			"application/pdf" => ContentType::PDF,
			"application/epub+zip" => ContentType::EPUB_ZIP,
			"application/zip" => ContentType::ZIP,
			"application/vnd.comicbook+zip" => ContentType::COMIC_ZIP,
			"application/vnd.rar" => ContentType::RAR,
			"application/vnd.comicbook-rar" => ContentType::COMIC_RAR,
			"image/png" => ContentType::PNG,
			"image/jpeg" => ContentType::JPEG,
			"image/webp" => ContentType::WEBP,
			"image/avif" => ContentType::AVIF,
			"image/gif" => ContentType::GIF,
			_ => ContentType::UNKNOWN,
		}
	}
}

impl std::fmt::Display for ContentType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ContentType::XHTML => write!(f, "application/xhtml+xml"),
			ContentType::XML => write!(f, "application/xml"),
			ContentType::HTML => write!(f, "text/html"),
			ContentType::PDF => write!(f, "application/pdf"),
			ContentType::EPUB_ZIP => write!(f, "application/epub+zip"),
			ContentType::ZIP => write!(f, "application/zip"),
			ContentType::COMIC_ZIP => write!(f, "application/vnd.comicbook+zip"),
			ContentType::RAR => write!(f, "application/vnd.rar"),
			ContentType::COMIC_RAR => write!(f, "application/vnd.comicbook-rar"),
			ContentType::PNG => write!(f, "image/png"),
			ContentType::JPEG => write!(f, "image/jpeg"),
			ContentType::WEBP => write!(f, "image/webp"),
			ContentType::AVIF => write!(f, "image/avif"),
			ContentType::GIF => write!(f, "image/gif"),
			ContentType::TXT => write!(f, "text/plain"),
			ContentType::UNKNOWN => write!(f, "unknown"),
		}
	}
}

impl From<ImageFormat> for ContentType {
	fn from(format: ImageFormat) -> Self {
		match format {
			ImageFormat::Jpeg => ContentType::JPEG,
			// TODO(339): Support JpegXl and Avif
			// ImageFormat::JpegXl => ContentType::JPEG,
			ImageFormat::Png => ContentType::PNG,
			ImageFormat::Webp => ContentType::WEBP,
			ImageFormat::Avif => ContentType::AVIF,
		}
	}
}

impl TryFrom<ContentType> for image::ImageFormat {
	type Error = CoreError;

	fn try_from(value: ContentType) -> Result<Self, Self::Error> {
		/// Internal helper function to reduce code duplication
		fn unsupported_error(unsupported_type: &str) -> CoreError {
			CoreError::InternalError(format!(
				"Cannot convert {} into image::ImageFormat, not supported.",
				unsupported_type
			))
		}

		// Match values that are compatible with the image crate. Other values should return
		// an error.
		match value {
			ContentType::PNG => Ok(image::ImageFormat::Png),
			ContentType::JPEG => Ok(image::ImageFormat::Jpeg),
			ContentType::WEBP => Ok(image::ImageFormat::WebP),
			ContentType::AVIF => Ok(image::ImageFormat::Avif),
			ContentType::GIF => Ok(image::ImageFormat::Gif),
			ContentType::XHTML => Err(unsupported_error("ContentType::XHTML")),
			ContentType::XML => Err(unsupported_error("ContentType::XML")),
			ContentType::HTML => Err(unsupported_error("ContentType::HTML")),
			ContentType::PDF => Err(unsupported_error("ContentType::PDF")),
			ContentType::EPUB_ZIP => Err(unsupported_error("ContentType::EPUB_ZIP")),
			ContentType::ZIP => Err(unsupported_error("ContentType::ZIP")),
			ContentType::COMIC_ZIP => Err(unsupported_error("ContentType::COMIC_ZIP")),
			ContentType::RAR => Err(unsupported_error("ContentType::RAR")),
			ContentType::COMIC_RAR => Err(unsupported_error("ContentType::COMIC_RAR")),
			ContentType::TXT => Err(unsupported_error("ContentType::TXT")),
			ContentType::UNKNOWN => Err(unsupported_error("ContentType::UNKNOWN")),
		}
	}
}
