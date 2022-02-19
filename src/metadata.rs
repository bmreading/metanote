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

use anyhow::{Context, Result};
use derive_builder::Builder;
use getset::{Getters, Setters};
use lofty::{Accessor, ItemKey, ItemValue, PictureType, Probe, Tag, TagItem};
use std::path::Path;

#[derive(Builder, Clone, Debug, Default, Getters, PartialEq, Setters)]
#[get = "pub"]
#[set = "pub"]
#[allow(dead_code)]
pub struct MetadataContainer {
    title: Option<String>,
    artist: Option<String>,
    album_artist: Option<String>,
    album: Option<String>,
    genre: Option<String>,
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
            .all(|c| c.genre().eq(&containers[0].genre()))
        {
            consolidated_container.genre = containers[0].genre.clone();
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
    description: Option<String>,
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
}

pub trait MetadataReadCapable {
    fn metadata(&self, path: &Path) -> Result<MetadataContainer>;
}

pub trait MetadataWriteCapable {
    fn write_metadata(&self, path: &Path, metadata: &MetadataContainer) -> Result<()>;
}

#[derive(Builder, Debug)]
pub struct MetadataAgent {}

impl MetadataAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl MetadataReadCapable for MetadataAgent {
    fn metadata(&self, path: &Path) -> Result<MetadataContainer> {
        let tagged_file = Probe::open(path)?.read(true)?;

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag.to_owned(),
            None => Tag::new(tagged_file.primary_tag_type()),
        };

        // Handle art
        let mut art = Vec::new();
        for art_element in tag.pictures() {
            let art_element = Art {
                description: art_element.description().map(|d| d.to_string()),
                mime_type: art_element.mime_type().to_string(),
                data: art_element.data().to_vec(),
            };

            art.push(art_element);
        }

        let art = match art.len() > 0 {
            true => Some(art),
            false => None,
        };

        // Debug log metadata info
        log::debug!(
            "Found tagged item at {}",
            path.to_str().context("failed to parse path as str")?
        );
        for item in tag.items() {
            log::debug!("{:?} - {:?}", item.key(), item.value());
        }

        Ok(MetadataContainerBuilder::default()
            .title(tag.title().map(|t| t.to_string()))
            .artist(tag.artist().map(|a| a.to_string()))
            .album(tag.album().map(|a| a.to_string()))
            .album_artist(tag.get_string(&ItemKey::AlbumArtist).map(|a| a.to_string()))
            .genre(tag.genre().map(|t| t.to_string()))
            .year(
                tag.get_string(&ItemKey::RecordingDate)
                    .map(|y| y.to_string()),
            )
            .art(art)
            .build()?)
    }
}

impl MetadataWriteCapable for MetadataAgent {
    fn write_metadata(&self, path: &Path, metadata: &MetadataContainer) -> Result<()> {
        let mut tagged_file = Probe::open(path)?.read(false)?;

        let tag = tagged_file
            .primary_tag_mut()
            .context("primary tag unable to be found")?;

        if let Some(title) = metadata.title() {
            tag.set_title(title.to_string())
        }

        if let Some(artist) = metadata.artist() {
            tag.set_artist(artist.to_string())
        }

        if let Some(album) = metadata.album() {
            tag.set_album(album.to_string())
        }

        if let Some(album_artist) = metadata.album_artist() {
            tag.insert_item(TagItem::new(
                ItemKey::AlbumArtist,
                ItemValue::Text(album_artist.to_string()),
            ));
        }

        if let Some(genre) = metadata.genre() {
            tag.set_genre(genre.to_string())
        }

        if let Some(year) = metadata.year() {
            tag.insert_item(TagItem::new(
                ItemKey::RecordingDate,
                ItemValue::Text(year.to_string()),
            ));
        }

        let mut pic_types = Vec::new();
        for existing_picture in tag.pictures() {
            pic_types.push(existing_picture.pic_type());
        }
        pic_types.dedup();

        // Remove existing art
        for pic_type in pic_types {
            tag.remove_picture_type(pic_type);
        }

        if let Some(art) = metadata.art() {
            for art_item in art {
                if art_item.description().is_some() {
                    let mut picture_type = PictureType::Other;
                    if art_item.description().as_ref().unwrap() == "cover" {
                        picture_type = PictureType::CoverFront
                    };

                    tag.push_picture(lofty::Picture::new_unchecked(
                        picture_type,
                        lofty::MimeType::from_str(art_item.mime_type()),
                        art_item.description().to_owned(),
                        art_item.data().to_vec(),
                    ));
                } else {
                    tag.push_picture(lofty::Picture::new_unchecked(
                        PictureType::Other,
                        lofty::MimeType::from_str(art_item.mime_type()),
                        None,
                        art_item.data().to_vec(),
                    ));
                }
            }
        }

        tag.save_to_path(path)?;

        Ok(())
    }
}
