// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Snapshot testing utilities.

use image::{DynamicImage, ImageBuffer, Rgba};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("Snapshot file not found: {0}")]
    NotFound(PathBuf),

    #[error("Snapshot mismatch: {0}")]
    Mismatch(PathBuf),

    #[error("Failed to save snapshot: {0}")]
    SaveError(#[from] std::io::Error),

    #[error("Failed to load image: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Failed to compare images: {0}")]
    ComparisonError(String),
}

/// Options for snapshot testing.
#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    pub tolerance: u8,
    pub padding: u32,
    pub background_color: [u8; 4],
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self { tolerance: 16, padding: 0, background_color: [0x29, 0x29, 0x29, 0xFF] }
    }
}

/// Snapshot manager for handling render snapshots.
pub struct SnapshotManager {
    snapshots_dir: PathBuf,
    options: SnapshotOptions,
}

impl SnapshotManager {
    pub fn new(snapshots_dir: PathBuf) -> Self {
        Self { snapshots_dir, options: SnapshotOptions::default() }
    }

    pub fn with_options(mut self, options: SnapshotOptions) -> Self {
        self.options = options;
        self
    }

    fn snapshot_path(&self, name: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}.png", name))
    }

    fn new_snapshot_path(&self, name: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}.new.png", name))
    }

    fn diff_path(&self, name: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}.diff.png", name))
    }

    pub fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.snapshots_dir)?;
        Ok(())
    }

    pub fn save(&self, name: &str, image: &DynamicImage) -> std::io::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.snapshot_path(name);
        image.save(&path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(path)
    }

    pub fn save_new(&self, name: &str, image: &DynamicImage) -> std::io::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.new_snapshot_path(name);
        image.save(&path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(path)
    }

    pub fn load(&self, name: &str) -> Result<DynamicImage, SnapshotError> {
        let path = self.snapshot_path(name);
        if !path.exists() {
            return Err(SnapshotError::NotFound(path));
        }
        Ok(image::open(&path).map_err(SnapshotError::ImageError)?)
    }

    pub fn compare(&self, name: &str, actual: &DynamicImage) -> Result<(), SnapshotError> {
        let expected = match self.load(name) {
            Ok(img) => img,
            Err(SnapshotError::NotFound(path)) => {
                self.save_new(name, actual)?;
                return Err(SnapshotError::NotFound(path));
            }
            Err(e) => return Err(e),
        };

        let expected: ImageBuffer<Rgba<u8>, Vec<u8>> = expected.to_rgba8();
        let actual_rgba: ImageBuffer<Rgba<u8>, Vec<u8>> = actual.to_rgba8();

        if expected.dimensions() != actual_rgba.dimensions() {
            self.save_new(name, actual)?;
            return Err(SnapshotError::Mismatch(self.snapshot_path(name)));
        }

        let (width, height) = expected.dimensions();
        let tolerance = self.options.tolerance as i32;

        for y in 0..height {
            for x in 0..width {
                let expected_pixel = expected.get_pixel(x, y);
                let actual_pixel = actual_rgba.get_pixel(x, y);

                let diff_r = (expected_pixel[0] as i32 - actual_pixel[0] as i32).abs();
                let diff_g = (expected_pixel[1] as i32 - actual_pixel[1] as i32).abs();
                let diff_b = (expected_pixel[2] as i32 - actual_pixel[2] as i32).abs();
                let diff_a = (expected_pixel[3] as i32 - actual_pixel[3] as i32).abs();

                if diff_r > tolerance
                    || diff_g > tolerance
                    || diff_b > tolerance
                    || diff_a > tolerance
                {
                    let mut diff = ImageBuffer::new(width, height);
                    for y in 0..height {
                        for x in 0..width {
                            let e = expected.get_pixel(x, y);
                            let a = actual_rgba.get_pixel(x, y);
                            let pixel = if e == a {
                                Rgba([128u8, 128u8, 128u8, 128u8])
                            } else {
                                Rgba([255u8, 0u8, 0u8, 255u8])
                            };
                            diff.put_pixel(x, y, pixel);
                        }
                    }
                    let diff_image = DynamicImage::ImageRgba8(diff);
                    diff_image
                        .save(self.diff_path(name))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    return Err(SnapshotError::Mismatch(self.snapshot_path(name)));
                }
            }
        }

        let _ = std::fs::remove_file(self.diff_path(name));
        Ok(())
    }

    pub fn bless(&self, name: &str, image: &DynamicImage) -> std::io::Result<PathBuf> {
        self.save(name, image)
    }
}

pub fn generate_diff(expected: &DynamicImage, actual: &DynamicImage) -> Option<DynamicImage> {
    let expected = expected.to_rgba8();
    let actual = actual.to_rgba8();

    if expected.dimensions() != actual.dimensions() {
        return None;
    }

    let (width, height) = expected.dimensions();
    let mut diff = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let e = expected.get_pixel(x, y);
            let a = actual.get_pixel(x, y);
            let pixel = if e == a {
                Rgba([128u8, 128u8, 128u8, 128u8])
            } else {
                Rgba([255u8, 0u8, 0u8, 255u8])
            };
            diff.put_pixel(x, y, pixel);
        }
    }

    Some(DynamicImage::ImageRgba8(diff))
}
