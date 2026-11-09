use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::ValidationError;
use crate::reported::Reported;
use crate::types::{ProductId, ProductImageId};

string_enum! {
    /// The format of a product image.
    pub enum ImageType {
        /// A JPEG image.
        Jpeg = "image/jpeg",
        /// A PNG image.
        Png = "image/png",
        /// A WebP image. The documentation only names JPEG and PNG, but Picqer accepts WebP too.
        Webp = "image/webp",
    }
}

impl ImageType {
    fn detect(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Some(Self::Jpeg)
        } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            Some(Self::Png)
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(Self::Webp)
        } else {
            None
        }
    }
}

/// An image of a product.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ProductImage {
    /// The image.
    pub idproduct_image: ProductImageId,
    /// The product.
    pub idproduct: ProductId,
    /// Where the image can be downloaded.
    pub url: Url,
    /// The format of the image.
    pub contenttype: Reported<ImageType>,
    /// The size of the image in bytes.
    pub size: i64,
}

/// An image to add to a product.
///
/// The format is recognised from the bytes and the image is encoded for the API, so the API cannot
/// refuse it for its format or its encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewProductImage {
    #[serde(skip)]
    image_type: ImageType,
    image: String,
}

impl NewProductImage {
    /// Checks that the bytes are a JPEG, PNG or WebP image.
    ///
    /// Only the format is recognised, not whether the whole image is valid.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, ValidationError> {
        let bytes = bytes.as_ref();
        let image_type = ImageType::detect(bytes)
            .ok_or_else(|| ValidationError::new("image", "must be a JPEG, PNG or WebP image"))?;
        Ok(Self {
            image_type,
            image: STANDARD.encode(bytes),
        })
    }

    /// The format of the image.
    pub fn image_type(&self) -> ImageType {
        self.image_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0];
    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0];
    const WEBP: &[u8] = b"RIFF\x1a\0\0\0WEBPVP8L";

    #[test]
    fn the_format_is_recognised_from_the_bytes() {
        assert_eq!(
            NewProductImage::new(PNG).unwrap().image_type(),
            ImageType::Png
        );
        assert_eq!(
            NewProductImage::new(JPEG).unwrap().image_type(),
            ImageType::Jpeg
        );
        assert_eq!(
            NewProductImage::new(WEBP).unwrap().image_type(),
            ImageType::Webp
        );
    }

    #[test]
    fn other_bytes_are_refused() {
        assert!(NewProductImage::new(b"hello").is_err());
        assert!(NewProductImage::new(b"GIF89a").is_err());
        assert!(NewProductImage::new(b"RIFF\0\0\0\0WAVE").is_err());
        assert!(NewProductImage::new([]).is_err());
    }

    #[test]
    fn the_image_is_sent_as_plain_base64() {
        let image = NewProductImage::new(PNG).unwrap();
        assert_eq!(
            serde_json::to_value(&image).unwrap(),
            serde_json::json!({"image": STANDARD.encode(PNG)})
        );
    }
}
