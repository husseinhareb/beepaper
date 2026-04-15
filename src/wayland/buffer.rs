//! Shared-memory buffer allocation for wallpaper pixels.

use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    os::fd::AsFd,
};

use tempfile::tempfile;
use wayland_client::{
    Dispatch, QueueHandle,
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
    },
};

use crate::error::WaylandError;

const BYTES_PER_PIXEL: u64 = 4;

/// A Wayland shared-memory buffer and its anonymous backing file.
#[derive(Debug)]
pub struct ShmBuffer {
    /// The Wayland buffer proxy.
    pub buffer: WlBuffer,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bytes per row.
    pub stride: i32,
    /// Backing file kept alive for the buffer lifetime.
    _file: File,
}

impl ShmBuffer {
    /// Create a shared-memory buffer from rendered XRGB pixel data.
    pub fn new<D>(
        shm: &WlShm,
        qh: &QueueHandle<D>,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<Self, WaylandError>
    where
        D: Dispatch<WlShmPool, ()> + Dispatch<WlBuffer, ()> + 'static,
    {
        let stride = stride_for_width(width)?;
        let byte_len = byte_len_for_buffer(width, height)?;

        if pixels.len() != byte_len {
            return Err(WaylandError::BufferSizeMismatch {
                expected: byte_len,
                actual: pixels.len(),
            });
        }

        let mut file = tempfile()?;
        file.set_len(byte_len as u64)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(pixels)?;

        let size = i32::try_from(byte_len).map_err(|_| WaylandError::BufferSizeOverflow)?;
        let width_i32 = i32::try_from(width).map_err(|_| WaylandError::BufferSizeOverflow)?;
        let height_i32 = i32::try_from(height).map_err(|_| WaylandError::BufferSizeOverflow)?;

        let pool = shm.create_pool(file.as_fd(), size, qh, ());
        let buffer = pool.create_buffer(
            0,
            width_i32,
            height_i32,
            stride,
            wl_shm::Format::Xrgb8888,
            qh,
            (),
        );
        pool.destroy();

        Ok(Self {
            buffer,
            width,
            height,
            stride,
            _file: file,
        })
    }
}

/// Return the byte stride for a buffer row.
pub fn stride_for_width(width: u32) -> Result<i32, WaylandError> {
    let stride = u64::from(width)
        .checked_mul(BYTES_PER_PIXEL)
        .ok_or(WaylandError::BufferSizeOverflow)?;

    i32::try_from(stride).map_err(|_| WaylandError::BufferSizeOverflow)
}

/// Return the total byte length for an XRGB buffer.
pub fn byte_len_for_buffer(width: u32, height: u32) -> Result<usize, WaylandError> {
    let byte_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL))
        .ok_or(WaylandError::BufferSizeOverflow)?;

    usize::try_from(byte_len).map_err(|_| WaylandError::BufferSizeOverflow)
}

#[cfg(test)]
mod tests {
    use super::{byte_len_for_buffer, stride_for_width};

    #[test]
    fn stride_calculation_uses_four_bytes_per_pixel() {
        assert_eq!(stride_for_width(1920).expect("stride"), 7680);
    }

    #[test]
    fn byte_length_matches_width_height_and_stride() {
        assert_eq!(byte_len_for_buffer(1920, 1080).expect("size"), 8_294_400);
    }

    #[test]
    fn overflowing_dimensions_are_rejected() {
        assert!(byte_len_for_buffer(u32::MAX, u32::MAX).is_err());
    }
}
