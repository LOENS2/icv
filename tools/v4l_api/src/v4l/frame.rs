use std::error::Error;
use std::io::Write;
use tokio_linux_video::types::{ColorSpace, Format, PixFormatMplane, Quantization};
use yuvutils_rs::{
    YuvBiPlanarImage, YuvConversionMode, YuvRange, YuvStandardMatrix, yuv_nv12_to_rgb,
};

pub struct Frame {
    format: Format,
    width: u32,
    height: u32,
    y_stride: u32,
    uv_stride: u32,
    y_size: usize,
    uv_size: usize,
    nv12: Vec<u8>,
}

impl Frame {
    pub fn from_capture(
        format: Format,
        nv12: Vec<u8>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let pix = format
            .try_ref::<PixFormatMplane>()
            .ok_or("Expected multi-planar video format")?;

        let width = pix.width();
        let height = pix.height();
        let pixel_format = pix.pixel_format();

        if pixel_format != tokio_linux_video::types::FourCc::Nv12
            && pixel_format != tokio_linux_video::types::FourCc::Nv21
        {
            return Err(format!("Unsupported format: {pixel_format} (expected NV12/NV21)").into());
        }
        if pixel_format == tokio_linux_video::types::FourCc::Nv21 {
            return Err("NV21 is not supported yet (expected NV12)".into());
        }

        let planes = pix.plane_fmt();
        let (y_stride, uv_stride, y_size, uv_size) = match planes {
            [y, uv, ..] => {
                let y_stride = y.bytes_per_line();
                let uv_stride = uv.bytes_per_line();
                let y_size = y.size_image().max(y_stride * height) as usize;
                let uv_size = uv.size_image().max(uv_stride * (height / 2)) as usize;
                (y_stride, uv_stride, y_size, uv_size)
            }
            [y] => {
                let y_stride = y.bytes_per_line();
                let uv_stride = y_stride;
                let y_size = (y_stride * height) as usize;
                let uv_size = y_size / 2;
                (y_stride, uv_stride, y_size, uv_size)
            }
            _ => return Err("No plane format information available".into()),
        };

        if nv12.len() < y_size + uv_size {
            return Err(format!(
                "Buffer too small: expected at least {}, got {}",
                y_size + uv_size,
                nv12.len()
            )
            .into());
        }

        Ok(Self {
            format,
            width,
            height,
            y_stride,
            uv_stride,
            y_size,
            uv_size,
            nv12,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn nv12_data(&self) -> &[u8] {
        &self.nv12[..self.y_size + self.uv_size]
    }

    pub fn into_tightly_packed_nv12(self) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let y_stride = self.y_stride as usize;
        let uv_stride = self.uv_stride as usize;
        let mut out = Vec::with_capacity(w * h * 3 / 2);
        let y_plane = &self.nv12[..self.y_size];
        for row in 0..h {
            let start = row * y_stride;
            out.extend_from_slice(&y_plane[start..start + w]);
        }
        let uv_plane = &self.nv12[self.y_size..self.y_size + self.uv_size];
        let uv_rows = h / 2;
        for row in 0..uv_rows {
            let start = row * uv_stride;
            out.extend_from_slice(&uv_plane[start..start + w]);
        }
        out
    }

    fn yuv_params(&self) -> (YuvRange, YuvStandardMatrix) {
        let pix = match self.format.try_ref::<PixFormatMplane>() {
            Some(p) => p,
            None => return (YuvRange::Full, YuvStandardMatrix::Bt709),
        };
        let range = match pix.quantization() {
            Quantization::LimRange => YuvRange::Limited,
            _ => YuvRange::Full,
        };
        let matrix = match pix.color_space() {
            ColorSpace::Smpte170M => YuvStandardMatrix::Bt601,
            ColorSpace::Bt2020 => YuvStandardMatrix::Bt2020,
            ColorSpace::Smpte240M => YuvStandardMatrix::Smpte240,
            _ => YuvStandardMatrix::Bt709,
        };
        (range, matrix)
    }

    /// Encode the frame as PNG by converting NV12 to RGB row-wise (2 rows at a time)
    /// and streaming scanlines to the PNG encoder. Avoids allocating a full RGB image.
    pub fn write_png<W: Write>(&self, writer: W) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (range, matrix) = self.yuv_params();
        let width = self.width;
        let height = self.height;
        let y_stride = self.y_stride as usize;
        let uv_stride = self.uv_stride as usize;
        let row_bytes = width as usize * 3;

        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        encoder.set_filter(png::Filter::Adaptive);
        let mut png_writer = encoder.write_header()?;
        let mut stream = png_writer.stream_writer()?;

        let y_plane = &self.nv12[..self.y_size];
        let uv_plane = &self.nv12[self.y_size..self.y_size + self.uv_size];
        let mut rgb = vec![0u8; row_bytes * 2];
        let mut uv_tmp: Vec<u8> = Vec::new();

        let mut y = 0u32;
        while y < height {
            let rows = if y + 1 < height { 2u32 } else { 1u32 };
            let chroma_rows = if rows == 2 { 1usize } else { 1usize };

            let y_off = y as usize * y_stride;
            let y_len = y_stride * rows as usize;
            let y_chunk = &y_plane[y_off..y_off + y_len];

            let uv_off = (y as usize / 2) * uv_stride;
            let uv_len = uv_stride * chroma_rows;
            let uv_chunk = &uv_plane[uv_off..uv_off + uv_len];

            // yuvutils-rs requires interleaved chroma len == stride * chroma_height.
            // If the driver pads, copy into an exact-sized buffer to satisfy validation.
            let uv_ref: &[u8] = if uv_chunk.len() == uv_stride * chroma_rows {
                uv_chunk
            } else {
                uv_tmp.clear();
                uv_tmp.resize(uv_stride * chroma_rows, 0);
                let copy = uv_chunk.len().min(uv_tmp.len());
                uv_tmp[..copy].copy_from_slice(&uv_chunk[..copy]);
                &uv_tmp
            };

            let yuv = YuvBiPlanarImage {
                y_plane: y_chunk,
                y_stride: self.y_stride,
                uv_plane: uv_ref,
                uv_stride: self.uv_stride,
                width,
                height: rows,
            };

            let out_len = row_bytes * rows as usize;
            yuv_nv12_to_rgb(
                &yuv,
                &mut rgb[..out_len],
                row_bytes as u32,
                range,
                matrix,
                YuvConversionMode::Balanced,
            )?;

            stream.write_all(&rgb[..out_len])?;
            y += rows;
        }

        stream.finish()?;
        png_writer.finish()?;
        Ok(())
    }
}
