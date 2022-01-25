// metadata.rs
//
// Copyright 2022 Brian Reading <brian.reading@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use audiotags::{Picture, Tag};
use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub struct MetadataContainer {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub images: Option<Vec<Image>>,
}

#[derive(Builder, Debug, Default)]
pub struct Image {
    description: String,
    mime_type: String,
    data: Vec<u8>,
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            mime_type: self.mime_type.clone(),
            data: self.data.clone(),
        }
    }
}

impl Image {
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

pub trait MetadataReadCapable {
    fn metadata(&self, path: &str) -> Result<MetadataContainer>;
}

pub trait MetadataWriteCapable {
    fn write_metadata(&self, path: &str, metadata: MetadataContainer) -> Result<()>;
}

#[derive(Builder, Debug, Default)]
pub struct MetadataAgent {}

impl MetadataReadCapable for MetadataAgent {
    fn metadata(&self, path: &str) -> Result<MetadataContainer> {
        let raw = Tag::default().read_from_path(&path)?;

        let images = if let Some(cover) = raw.album_cover() {
            Some(vec![cover.to_image_with_description("cover")])
        } else {
            None
        };

        Ok(MetadataContainerBuilder::default()
            .title(raw.title().map(|t| t.to_string()))
            .artist(raw.artist().map(|a| a.to_string()))
            .album_artist(raw.album_artist().map(|a| a.to_string()))
            .album(raw.album().map(|a| a.title.to_string()))
            .year(raw.year().map(|a| a.to_string()))
            .images(images)
            .build()?)
    }
}

pub trait PictureExt<Picture> {
    fn to_image_with_description(&self, description: &str) -> Image;
}

impl<'a> PictureExt<Picture<'a>> for Picture<'a> {
    fn to_image_with_description(&self, description: &str) -> Image {
        Image {
            description: description.to_string(),
            mime_type: self.mime_type.try_into().unwrap(),
            data: self.data.to_vec(),
        }
    }
}
