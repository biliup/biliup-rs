use crate::uploader::bilibili::Studio;
use async_trait::async_trait;
use sqlx::FromRow;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub type DynUploadStreamersRepository = Arc<dyn UploadStreamersRepository + Send + Sync>;

#[async_trait]
pub trait UploadStreamersRepository {
    async fn create_streamer(&self, studio: Studio) -> anyhow::Result<StudioEntity>;
    async fn get_streamers(&self) -> anyhow::Result<Vec<StudioEntity>>;
    async fn get_streamer_by_id(&self, id: u32) -> anyhow::Result<StudioEntity>;
}

#[derive(FromRow,Serialize, Deserialize)]
pub struct StudioEntity {
    pub id: u32,
    pub copyright: u8,
    pub source: String,
    pub tid: u16,
    pub cover: String,
    pub title: String,
    pub desc: String,
    pub dynamic: String,
    pub tag: String,
    pub dtime: Option<u32>,
    pub interactive: u8,
    pub mission_id: Option<u32>,
    pub dolby: u8,
    pub lossless_music: u8,
    pub no_reprint: Option<u8>,
    pub up_selection_reply: bool,
    pub up_close_reply: bool,
    pub up_close_danmu: bool,
    pub open_elec: Option<u8>,
}

impl StudioEntity {
    pub fn into_dto(self) -> Studio {
        Studio {
            copyright: self.copyright,
            source: self.source,
            tid: self.tid,
            cover: self.cover,
            title: self.title,
            desc_format_id: 0,
            desc: self.desc,
            dynamic: self.dynamic,
            subtitle: Default::default(),
            tag: self.tag,
            videos: vec![],
            dtime: self.dtime,
            open_subtitle: false,
            interactive: self.interactive,
            mission_id: self.mission_id,
            dolby: self.dolby,
            lossless_music: self.lossless_music,
            no_reprint: self.no_reprint,
            aid: None,
            up_selection_reply: self.up_selection_reply,
            up_close_reply: self.up_close_reply,
            up_close_danmu: self.up_close_danmu,
            open_elec: self.open_elec,
        }
    }

    // pub fn into_profile(self, following: bool) -> ProfileDto {
    //     ProfileDto {
    //         username: self.username,
    //         bio: self.bio,
    //         image: self.image,
    //         following,
    //     }
    // }
}
