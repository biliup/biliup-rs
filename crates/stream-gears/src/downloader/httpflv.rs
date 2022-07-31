use crate::downloader::util::Segment;
use crate::flv_parser::{
    aac_audio_packet_header, avc_video_packet_header, script_data, tag_data, tag_header,
    AACPacketType, AVCPacketType, CodecId, FrameType, SoundFormat, TagData, TagHeader,
};
use crate::flv_writer::{FlvFile, FlvTag, TagDataHeader};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use nom::{Err, IResult};
use std::io::{ErrorKind, Read};
use std::time::Duration;
use tracing::{info, warn};

pub fn download<T: Read>(connection: Connection<T>, file_name: &str, segment: Segment) {
    match parse_flv(connection, file_name, segment) {
        Ok(_) => {
            info!("Done... {file_name}");
        }
        Err(e) => {
            warn!("{e}")
        }
    }
}

fn parse_flv<T: Read>(
    mut connection: Connection<T>,
    file_name: &str,
    mut segment: Segment,
) -> core::result::Result<(), crate::error::Error> {
    let mut flv_tags_cache: Vec<(TagHeader, Bytes, Bytes)> = Vec::new();

    let _previous_tag_size = connection.read_frame(4)?;
    // let mut rdr = Cursor::new(previous_tag_size);
    // println!("{}", rdr.read_u32::<BigEndian>().unwrap());
    // let file = std::fs::File::create(format!("{file_name}_flv.json"))?;
    // let mut writer = BufWriter::new(file);
    // flv_writer::to_json(&mut writer, &header)?;

    let mut out = FlvFile::new(file_name)?;
    let mut downloaded_size = 9 + 4;
    let mut on_meta_data = None;
    let mut aac_sequence_header = None;
    let mut h264_sequence_header: Option<(TagHeader, Bytes, Bytes)> = None;
    let mut prev_timestamp = 0;
    let mut create_new = false;
    loop {
        let tag_header_bytes = connection.read_frame(11)?;
        if tag_header_bytes.is_empty() {
            // let mut rdr = Cursor::new(tag_header_bytes);
            // println!("{}", rdr.read_u32::<BigEndian>().unwrap());
            break;
        }

        let (_, tag_header) = map_parse_err(tag_header(&tag_header_bytes), "tag header")?;
        // write_tag_header(&mut out, &tag_header)?;

        let bytes = connection.read_frame(tag_header.data_size as usize)?;
        let previous_tag_size = connection.read_frame(4)?;
        // out.write(&bytes)?;
        let (i, flv_tag_data) = map_parse_err(
            tag_data(tag_header.tag_type, tag_header.data_size as usize)(&bytes),
            "tag data",
        )?;
        let flv_tag = match flv_tag_data {
            TagData::Audio(audio_data) => {
                let packet_type = if audio_data.sound_format == SoundFormat::AAC {
                    let (_, packet_header) = aac_audio_packet_header(audio_data.sound_data)
                        .expect("Error in parsing aac audio packet header.");
                    if packet_header.packet_type == AACPacketType::SequenceHeader {
                        if aac_sequence_header.is_some() {
                            warn!("Unexpected aac sequence header tag. {tag_header:?}");
                            // panic!("Unexpected aac_sequence_header tag.");
                            // create_new = true;
                        }
                        aac_sequence_header =
                            Some((tag_header, bytes.clone(), previous_tag_size.clone()))
                    }
                    Some(packet_header.packet_type)
                } else {
                    None
                };

                FlvTag {
                    header: tag_header,
                    data: TagDataHeader::Audio {
                        sound_format: audio_data.sound_format,
                        sound_rate: audio_data.sound_rate,
                        sound_size: audio_data.sound_size,
                        sound_type: audio_data.sound_type,
                        packet_type,
                    },
                }
            }
            TagData::Video(video_data) => {
                let (packet_type, composition_time) = if CodecId::H264 == video_data.codec_id {
                    let (_, avc_video_header) = avc_video_packet_header(video_data.video_data)
                        .expect("Error in parsing avc video packet header.");
                    if avc_video_header.packet_type == AVCPacketType::SequenceHeader {
                        h264_sequence_header = match h264_sequence_header {
                            None => Some((tag_header, bytes.clone(), previous_tag_size.clone())),
                            Some((_, binary_data, _)) => {
                                warn!("Unexpected h264 sequence header tag. {tag_header:?}");
                                // panic!("Unexpected h264 sequence header tag.");
                                if bytes != binary_data {
                                    create_new = true;
                                    warn!("Different h264 sequence header tag. {tag_header:?}");
                                }
                                Some((tag_header, bytes.clone(), previous_tag_size.clone()))
                            }
                        };
                    }
                    (
                        Some(avc_video_header.packet_type),
                        Some(avc_video_header.composition_time),
                    )
                } else {
                    (None, None)
                };

                FlvTag {
                    header: tag_header,
                    data: TagDataHeader::Video {
                        frame_type: video_data.frame_type,
                        codec_id: video_data.codec_id,
                        packet_type,
                        composition_time,
                    },
                }
            }
            TagData::Script => {
                let (_, tag_data) = script_data(i).expect("Error in parsing script tag.");
                if on_meta_data.is_some() {
                    warn!("Unexpected script tag. {tag_header:?}");
                    // create_new = true;

                    // panic!("Unexpected script tag.");
                }
                on_meta_data = Some((tag_header, bytes.clone(), previous_tag_size.clone()));

                let flv_tag = FlvTag {
                    header: tag_header,
                    data: TagDataHeader::Script(tag_data),
                };
                flv_tag
            }
        };
        match &flv_tag {
            FlvTag {
                data:
                    TagDataHeader::Video {
                        frame_type: FrameType::Key,
                        ..
                    },
                ..
            } => {
                if segment.needed(
                    downloaded_size,
                    Duration::from_millis(flv_tag.header.timestamp as u64),
                ) {
                    // let new_file_name = format_filename(file_name);
                    downloaded_size = 9 + 4;
                    out = FlvFile::new(file_name)?;
                    let on_meta_data = on_meta_data.as_ref().expect("on_meta_data does not exist");
                    // onMetaData
                    out.write_tag(&on_meta_data.0, &on_meta_data.1, &on_meta_data.2)?;
                    // AACSequenceHeader
                    let aac_sequence_header = aac_sequence_header
                        .as_ref()
                        .expect("aac_sequence_header does not exist");
                    out.write_tag(
                        &aac_sequence_header.0,
                        &aac_sequence_header.1,
                        &aac_sequence_header.2,
                    )?;
                    // H264SequenceHeader
                    let h264_sequence_header = h264_sequence_header
                        .as_ref()
                        .expect("h264_sequence_header does not exist");
                    out.write_tag(
                        &h264_sequence_header.0,
                        &h264_sequence_header.1,
                        &h264_sequence_header.2,
                    )?;
                    info!("{} splitting.{segment:?}", out.name);
                }

                for (tag_header, flv_tag_data, previous_tag_size_bytes) in &flv_tags_cache {
                    if tag_header.timestamp < prev_timestamp {
                        warn!("Non-monotonous DTS in output stream; previous: {prev_timestamp}, current: {};", tag_header.timestamp);
                    }
                    out.write_tag(tag_header, flv_tag_data, previous_tag_size_bytes)?;
                    // out.write_tag_header( tag_header)?;
                    // out.write(flv_tag_data)?;
                    // out.write(previous_tag_size_bytes)?;
                    downloaded_size += (11 + tag_header.data_size + 4) as u64;
                    prev_timestamp = tag_header.timestamp
                    // println!("{downloaded_size}");
                }
                flv_tags_cache.clear();
                if create_new {
                    // let new_file_name = format_filename(file_name);
                    out = FlvFile::new(file_name)?;
                    // let on_meta_data = on_meta_data.as_ref().unwrap();
                    // flv_tags_cache.push(on_meta_data)
                    // onMetaData
                    let on_meta_data = on_meta_data.as_ref().expect("on_meta_data does not exist");
                    out.write_tag(&on_meta_data.0, &on_meta_data.1, &on_meta_data.2)?;
                    // AACSequenceHeader
                    let aac_sequence_header = aac_sequence_header
                        .as_ref()
                        .expect("aac_sequence_header does not exist");
                    out.write_tag(
                        &aac_sequence_header.0,
                        &aac_sequence_header.1,
                        &aac_sequence_header.2,
                    )?;
                    create_new = false;
                    info!("{} splitting.", out.name);
                }
                flv_tags_cache.push((tag_header, bytes.clone(), previous_tag_size.clone()));
            }
            _ => {
                flv_tags_cache.push((tag_header, bytes.clone(), previous_tag_size.clone()));
            }
        }
        // flv_writer::to_json(&mut writer, &flv_tag)?;
    }
    Ok(())
}

// fn is_splitting(
//     flv_tag: FlvTag,
//     segment: &Segment,
//     first_tag_time: &mut u32,
//     downloaded_size: &mut u64,
// ) -> bool {
//     match segment {
//         Segment::Time(duration, _) => {
//             if duration
//                 <= &Duration::from_millis((flv_tag.header.timestamp - *first_tag_time) as u64)
//             {
//                 *first_tag_time = flv_tag.header.timestamp;
//                 true
//             } else {
//                 false
//             }
//         }
//         Segment::Size(file_size, _) => {
//             if *downloaded_size >= *file_size {
//                 *downloaded_size = 9 + 4;
//                 true
//             } else {
//                 false
//             }
//         }
//     }
// }

pub fn map_parse_err<'a, T>(
    i_result: IResult<&'a [u8], T>,
    msg: &str,
) -> core::result::Result<(&'a [u8], T), crate::error::Error> {
    match i_result {
        Ok((i, res)) => Ok((i, res)),
        Err(nom::Err::Incomplete(needed)) => {
            Err(crate::error::Error::NomIncomplete(msg.to_string(), needed))
        }
        Err(Err::Error(e)) => {
            panic!("parse {msg} err: {e:?}")
        }
        Err(Err::Failure(f)) => {
            panic!("{msg} Failure: {f:?}")
        }
    }
}

pub struct Connection<T> {
    resp: T,
    buffer: BytesMut,
}

impl<T: Read> Connection<T> {
    pub fn new(resp: T) -> Connection<T> {
        Connection {
            resp,
            buffer: BytesMut::with_capacity(8 * 1024),
        }
    }

    pub fn read_frame(&mut self, chunk_size: usize) -> std::io::Result<Bytes> {
        let mut buf = [0u8; 8 * 1024];
        loop {
            if chunk_size <= self.buffer.len() {
                let bytes = Bytes::copy_from_slice(&self.buffer[..chunk_size]);
                self.buffer.advance(chunk_size as usize);
                return Ok(bytes);
            }
            // BytesMut::with_capacity(0).deref_mut()
            // tokio::fs::File::open("").read()
            let n = match self.resp.read(&mut buf) {
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            if n == 0 {
                return Ok(self.buffer.split().freeze());
            }
            self.buffer.put_slice(&buf[..n]);
        }
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use bytes::{Buf, BufMut, BytesMut};

    #[test]
    fn byte_it_works() -> Result<()> {
        let mut bb = bytes::BytesMut::with_capacity(10);
        println!("chunk {:?}", bb.chunk());
        println!("capacity {}", bb.capacity());
        bb.put(&b"hello"[..]);
        println!("chunk {:?}", bb.chunk());
        println!("remaining {}", bb.remaining());
        bb.advance(5);
        println!("capacity {}", bb.capacity());
        println!("chunk {:?}", bb.chunk());
        println!("remaining {}", bb.remaining());
        bb.put(&b"hello"[..]);
        bb.put(&b"hello"[..]);
        println!("chunk {:?}", bb.chunk());
        println!("capacity {}", bb.capacity());
        println!("remaining {}", bb.remaining());

        let mut buf = BytesMut::with_capacity(11);
        buf.put(&b"hello world"[..]);

        let other = buf.split();
        // buf.advance_mut()

        assert!(buf.is_empty());
        assert_eq!(0, buf.capacity());
        assert_eq!(11, other.capacity());
        assert_eq!(other, b"hello world"[..]);

        Ok(())
    }

    #[test]
    fn it_works() -> Result<()> {
        // download(
        //     "test.flv")?;
        Ok(())
    }
}
