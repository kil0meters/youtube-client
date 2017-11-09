//  Copyright (C) 2017  Kil0meters
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gtk;
use gdk_pixbuf::Pixbuf;
use glib;
use gtk::prelude::*;

use std::sync::mpsc::channel;
use std::path::Path;
use std::thread;

use lib::utils::get_config_dir;

use ui::THUMBNAILS;
use lib::downloader;

// http://gtk-rs.org/tuto/closures
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

// Caches image at $HOME/.config/$NAME_NOCAPS/cache/images/$ID.jpg
pub fn load_thumbnails(images: Vec<gtk::Image>, ids: Vec<String>) {
    let (tx, rx) = channel();
    THUMBNAILS.with(move |thumbnails| {
        *thumbnails.borrow_mut() = Some((images, rx));
    });
    thread::spawn(move || {
        let cache_dir = format!("{}/cache/images", get_config_dir());
        for (i, id) in ids.iter().enumerate() {
            let file = format!("{}.jpg", id);
            let file_dir = format!("{}/{}", &cache_dir, &file);
            if !Path::new(&file_dir).is_file() {
                let url = format!("https://i.ytimg.com/vi/{}/mqdefault.jpg", id);
                downloader::download_to(&cache_dir, &file, &url);
            }
            match tx.send(Some(file_dir.clone())) {
                Ok(()) => (),
                Err(e) => eprintln!("Could not send data to thread: {:?}", e),
            };
            glib::idle_add(move || {
                THUMBNAILS.with(|thumbnails| if let Some((ref images, ref rx)) =
                    *thumbnails.borrow() {
                    if let Ok(file_dir) = rx.try_recv() {
                        let file_dir = file_dir.unwrap();
                        let pixbuf = Pixbuf::new_from_file_at_size(&file_dir, 240, 135).unwrap();
                        images[i].set_from_pixbuf(&pixbuf);
                    }
                });
                glib::Continue(false)
            });
        }
    });
}
