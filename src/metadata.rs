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
use audiotags::Tag;
use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub struct MetadataContainer {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
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

        Ok(MetadataContainerBuilder::default()
            .title(raw.title().map(|t| t.to_string()))
            .artist(raw.artist().map(|a| a.to_string()))
            .album_artist(raw.album_artist().map(|a| a.to_string()))
            .album(raw.album().map(|a| a.title.to_string()))
            .year(raw.year().map(|a| a.to_string()))
            .build()?)
    }
}
