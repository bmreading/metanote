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

use anyhow::{Error, Result};
use audiotags::{Picture, Tag};
use derive_builder::Builder;
use getset::Getters;
use std::path::Path;

#[derive(Builder, Debug, Default, Getters, PartialEq)]
#[get = "pub"]
#[allow(dead_code)]
pub struct MetadataContainer {
    title: Option<String>,
    artist: Option<String>,
    album_artist: Option<String>,
    album: Option<String>,
    year: Option<String>,
    art: Option<Vec<Art>>,
}

#[derive(Clone, Debug, Getters, PartialEq)]
#[get = "pub"]
#[allow(dead_code)]
pub struct Art {
    description: String,
    mime_type: String,
    data: Vec<u8>,
}

pub trait MetadataReadCapable {
    fn metadata(&self, path: &Path) -> Result<MetadataContainer>;
}

pub trait MetadataWriteCapable {
    fn write_metadata(&self, path: &Path, metadata: MetadataContainer) -> Result<()>;
}

#[derive(Builder, Debug, Default)]
pub struct MetadataAgent {}

impl MetadataReadCapable for MetadataAgent {
    fn metadata(&self, path: &Path) -> Result<MetadataContainer> {
        // Test this path before we try it, because
        // the backend panics on bad paths
        if !path.is_file() {
            return Err(Error::msg("bad path"));
        }

        let raw = Tag::default().read_from_path(path)?;

        let art = if let Some(cover) = raw.album_cover() {
            Some(vec![cover.to_art_with_description("cover")])
        } else {
            None
        };

        Ok(MetadataContainerBuilder::default()
            .title(raw.title().map(|t| t.to_string()))
            .artist(raw.artist().map(|a| a.to_string()))
            .album_artist(raw.album_artist().map(|a| a.to_string()))
            .album(raw.album().map(|a| a.title.to_string()))
            .year(raw.year().map(|a| a.to_string()))
            .art(art)
            .build()?)
    }
}

pub trait PictureExt<Picture> {
    fn to_art_with_description(&self, description: &str) -> Art;
}

impl<'a> PictureExt<Picture<'a>> for Picture<'a> {
    fn to_art_with_description(&self, description: &str) -> Art {
        Art {
            description: description.to_string(),
            mime_type: self.mime_type.try_into().unwrap(),
            data: self.data.to_vec(),
        }
    }
}
