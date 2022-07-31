use crate::downloader::util;
use crate::flv_parser::{
    AACPacketType, AVCPacketType, CodecId, FrameType, ScriptData, SoundFormat, SoundRate,
    SoundSize, SoundType, TagHeader,
};
use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use tracing::error;

const FLV_HEADER: [u8; 9] = [
    0x46, // 'F'
    0x4c, //'L'
    0x56, //'V'
    0x01, //version
    0x05, //00000101  audio tag  and video tag
    0x00, 0x00, 0x00, 0x09, //flv header size
]; // 9

pub struct FlvFile {
    pub buf_writer: BufWriter<File>,
    pub name: String,
}

impl FlvFile {
    pub fn new(file_name: &str) -> std::io::Result<Self> {
        let file_name = util::format_filename(file_name);
        let out =
            File::create(format!("{file_name}.flv.part")).expect("Unable to create flv file.");
        let mut buf_writer = BufWriter::new(out);
        buf_writer.write_all(&FLV_HEADER)?;
        Self::write_previous_tag_size(&mut buf_writer, 0)?;
        Ok(Self {
            buf_writer,
            name: file_name,
        })
    }

    pub fn write_tag(
        &mut self,
        tag_header: &TagHeader,
        body: &[u8],
        previous_tag_size: &[u8],
    ) -> std::io::Result<usize> {
        self.write_tag_header(tag_header)?;
        self.buf_writer.write_all(body)?;
        self.buf_writer.write(previous_tag_size)
    }

    pub fn write_tag_header(&mut self, tag_header: &TagHeader) -> std::io::Result<()> {
        self.buf_writer.write_u8(tag_header.tag_type as u8)?;
        self.buf_writer
            .write_u24::<BigEndian>(tag_header.data_size)?;
        self.buf_writer
            .write_u24::<BigEndian>(tag_header.timestamp & 0xffffff)?;
        let timestamp_ext = (tag_header.timestamp >> 24 & 0xff) as u8;
        self.buf_writer.write_u8(timestamp_ext)?;
        self.buf_writer.write_u24::<BigEndian>(tag_header.stream_id)
    }

    pub fn write_previous_tag_size(
        writer: &mut impl Write,
        previous_tag_size: u32,
    ) -> std::io::Result<usize> {
        writer.write(&previous_tag_size.to_be_bytes())
    }
}

impl Drop for FlvFile {
    fn drop(&mut self) {
        std::fs::rename(
            format!("{}.flv.part", self.name),
            format!("{}.flv", self.name),
        )
        .unwrap_or_else(|e| error!("{e}"))
    }
}

// pub fn create_flv_file(file_name: &str) -> std::io::Result<impl Write> {
//
// }

#[derive(Debug, PartialEq, Serialize)]
pub struct FlvTag<'a> {
    pub header: TagHeader,
    pub data: TagDataHeader<'a>,
}

pub fn to_json<T: ?Sized + Serialize>(mut writer: impl Write, t: &T) -> std::io::Result<usize> {
    serde_json::to_writer(&mut writer, t)?;
    writer.write("\n".as_ref())
}

#[derive(Debug, PartialEq, Serialize)]
pub enum TagDataHeader<'a> {
    Audio {
        sound_format: SoundFormat,
        sound_rate: SoundRate,
        sound_size: SoundSize,
        sound_type: SoundType,
        packet_type: Option<AACPacketType>,
    },
    Video {
        frame_type: FrameType,
        codec_id: CodecId,
        packet_type: Option<AVCPacketType>,
        composition_time: Option<i32>,
    },
    Script(ScriptData<'a>),
}
