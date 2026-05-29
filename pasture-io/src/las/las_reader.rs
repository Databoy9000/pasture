use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};
use std::{io::SeekFrom, path::Path};

use anyhow::Result;
use las_rs::Header;

use crate::base::{PointReader, SeekToPoint};
use pasture_core::{containers::BorrowedMutBuffer, layout::PointLayout, meta::Metadata};

use super::{LASMetadata, LASReaderBase, RawLASReader, RawLAZReader, path_is_compressed_las_file};

pub enum LASReaderFlavor<T: Read + Seek + Send> {
    LAS(RawLASReader<T>),
    LAZ(RawLAZReader<T>),
}

impl<T: Read + Seek + Send> LASReaderFlavor<T> {
    pub fn remaining_points(&self) -> usize {
        match self {
            LASReaderFlavor::LAS(reader) => reader.remaining_points(),
            LASReaderFlavor::LAZ(reader) => reader.remaining_points(),
        }
    }

    pub fn header(&self) -> &Header {
        match self {
            LASReaderFlavor::LAS(reader) => reader.header(),
            LASReaderFlavor::LAZ(reader) => reader.header(),
        }
    }
}

impl<T: Read + Seek + Send> PointReader for LASReaderFlavor<T> {
    fn read_into<'b, 'c, B: BorrowedMutBuffer<'b>>(
        &mut self,
        point_buffer: &'c mut B,
        count: usize,
    ) -> Result<usize>
    where
        'b: 'c,
    {
        match self {
            LASReaderFlavor::LAS(reader) => reader.read_into(point_buffer, count),
            LASReaderFlavor::LAZ(reader) => reader.read_into(point_buffer, count),
        }
    }

    fn get_metadata(&self) -> &dyn Metadata {
        match self {
            LASReaderFlavor::LAS(reader) => reader.get_metadata(),
            LASReaderFlavor::LAZ(reader) => reader.get_metadata(),
        }
    }

    fn get_default_point_layout(&self) -> &PointLayout {
        match self {
            LASReaderFlavor::LAS(reader) => reader.get_default_point_layout(),
            LASReaderFlavor::LAZ(reader) => reader.get_default_point_layout(),
        }
    }
}

impl<T: Read + Seek + Send> SeekToPoint for LASReaderFlavor<T> {
    fn seek_point(&mut self, position: SeekFrom) -> Result<usize> {
        match self {
            LASReaderFlavor::LAS(reader) => reader.seek_point(position),
            LASReaderFlavor::LAZ(reader) => reader.seek_point(position),
        }
    }
}

/// `PointReader` implementation for LAS/LAZ files
pub struct LASReader<R: Read + Seek + Send> {
    raw_reader: LASReaderFlavor<R>,
}

impl LASReader<BufReader<File>> {
    /// Creates a new `LASReader` by opening the file at the given `path`. Tries to determine whether
    /// the file is compressed from the file extension (i.e. files with extension `.laz` are assumed to be
    /// compressed). If `point_layout_matches_memory_layout`
    /// is `true`, the reader will return point data with a `PointLayout` that exactly matches the binary
    /// layout of the LAS point records. See [`point_layout_from_las_point_format`] for more information.
    ///
    /// # Errors
    ///
    /// If `path` does not exist, cannot be opened or does not point to a valid LAS/LAZ file, an error is returned.
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        point_layout_matches_memory_layout: bool,
    ) -> Result<LASReader<BufReader<File>>> {
        let is_compressed = path_is_compressed_las_file(path.as_ref())?;
        let file = BufReader::new(File::open(path)?);
        Self::from_read(file, is_compressed, point_layout_matches_memory_layout)
    }
}

impl<R: Read + Seek + Send> LASReader<R> {
    /// Creates a new `LASReader` from the given `read`. This method has to know whether
    /// the `read` points to a compressed LAZ file or a regular LAS file. If `point_layout_matches_memory_layout`
    /// is `true`, the reader will return point data with a `PointLayout` that exactly matches the binary
    /// layout of the LAS point records. See [`point_layout_from_las_point_format`] for more information.
    ///
    /// # Errors
    ///
    /// If the given `Read` does not represent a valid LAS/LAZ file, an error is returned.
    pub fn from_read(
        read: R,
        is_compressed: bool,
        point_layout_matches_memory_layout: bool,
    ) -> Result<Self> {
        let raw_reader = if is_compressed {
            LASReaderFlavor::LAZ(RawLAZReader::from_read(
                read,
                point_layout_matches_memory_layout,
            )?)
        } else {
            LASReaderFlavor::LAS(RawLASReader::from_read(
                read,
                point_layout_matches_memory_layout,
            )?)
        };
        Ok(Self { raw_reader })
    }

    pub fn remaining_points(&self) -> usize {
        self.raw_reader.remaining_points()
    }

    /// Returns the LAS header for the associated `LASReader`
    pub fn header(&self) -> &Header {
        self.raw_reader.header()
    }

    /// Returns the LAS metadata for the associated `LASReader`
    pub fn las_metadata(&self) -> &LASMetadata {
        match &self.raw_reader {
            LASReaderFlavor::LAS(reader) => reader.las_metadata(),
            LASReaderFlavor::LAZ(reader) => reader.las_metadata(),
        }
    }
}

impl<R: Read + Seek + Send> PointReader for LASReader<R> {
    fn get_metadata(&self) -> &dyn Metadata {
        self.raw_reader.get_metadata()
    }

    fn get_default_point_layout(&self) -> &PointLayout {
        self.raw_reader.get_default_point_layout()
    }

    fn read_into<'b, 'c, B: BorrowedMutBuffer<'b>>(
        &mut self,
        point_buffer: &'c mut B,
        count: usize,
    ) -> Result<usize>
    where
        'b: 'c,
    {
        self.raw_reader.read_into(point_buffer, count)
    }
}

impl<R: Read + Seek + Send> SeekToPoint for LASReader<R> {
    fn seek_point(&mut self, position: SeekFrom) -> Result<usize> {
        self.raw_reader.seek_point(position)
    }
}
