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

use anyhow::{Context, Error, Result};
use audiotags::{MimeType, Picture, Tag};
use derive_builder::Builder;
use getset::Getters;
use std::path::Path;

#[derive(Builder, Clone, Debug, Default, Getters, PartialEq)]
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

impl MetadataContainer {
    /// Returns a single MetadataContainer consolidated with matching fields.
    /// Non-matching fields are None
    pub fn merge(containers: &[Self]) -> Self {
        let mut consolidated_container = Self::default();

        if containers
            .iter()
            .all(|c| c.title().eq(&containers[0].title))
        {
            consolidated_container.title = containers[0].title.clone();
        }

        if containers
            .iter()
            .all(|c| c.artist().eq(&containers[0].artist()))
        {
            consolidated_container.artist = containers[0].artist.clone();
        }

        if containers
            .iter()
            .all(|c| c.album_artist.eq(&containers[0].album_artist()))
        {
            consolidated_container.album_artist = containers[0].album_artist.clone();
        }

        if containers
            .iter()
            .all(|c| c.album().eq(&containers[0].album()))
        {
            consolidated_container.album = containers[0].album.clone();
        }

        if containers
            .iter()
            .all(|c| c.year().eq(&containers[0].year()))
        {
            consolidated_container.year = containers[0].year.clone();
        }

        if containers.iter().all(|c| c.art().eq(&containers[0].art())) {
            consolidated_container.art = containers[0].art.clone();
        }

        consolidated_container
    }
}

#[derive(Clone, Debug, Getters, PartialEq)]
#[get = "pub"]
#[allow(dead_code)]
pub struct Art {
    description: String,
    mime_type: String,
    data: Vec<u8>,
}

impl Art {
    /// Returns a GTK Picture widget
    pub fn to_picture_widget(&self) -> gtk::Picture {
        let bytes = gtk::glib::Bytes::from(self.data());
        let stream = gtk::gio::MemoryInputStream::from_bytes(&bytes);
        let pixbuf =
            gtk::gdk_pixbuf::Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE).unwrap();
        gtk::Picture::for_pixbuf(&pixbuf)
    }

    fn to_picture(&self) -> Result<Picture> {
        Ok(Picture::new(
            &self.data,
            MimeType::try_from(self.mime_type().as_str())?,
        ))
    }
}

pub trait MetadataReadCapable {
    fn metadata(&self, path: &Path) -> Result<MetadataContainer>;
}

pub trait MetadataWriteCapable {
    fn write_metadata(&self, path: &Path, metadata: &MetadataContainer) -> Result<()>;
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

impl MetadataWriteCapable for MetadataAgent {
    fn write_metadata(&self, path: &Path, metadata: &MetadataContainer) -> Result<()> {
        // Test this path before we try it, because
        // the backend panics on bad paths
        if !path.is_file() {
            return Err(Error::msg("bad path"));
        }

        let empty_value = String::from("");

        let album = match metadata.art() {
            Some(art) => {
                let cover = art[0].to_picture()?;
                audiotags::Album::with_all(
                    &metadata.album().as_ref().unwrap_or(&empty_value),
                    &metadata.album_artist().as_ref().unwrap_or(&empty_value),
                    cover,
                )
            }
            None => audiotags::Album::with_title(metadata.album().as_ref().unwrap_or(&empty_value))
                .and_artist(&metadata.album_artist().as_ref().unwrap_or(&empty_value)),
        };

        let mut file = Tag::new().read_from_path(path)?;
        file.set_title(metadata.title().as_ref().unwrap_or(&empty_value));
        file.set_artist(metadata.artist().as_ref().unwrap_or(&empty_value));
        file.set_album(album);

        if let Some(year) = &metadata.year {
            file.set_year(year.parse::<i32>().expect("cannot parse year"));
        }

        file.write_to_path(path.to_str().context("bad path")?)?;

        Ok(())
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
