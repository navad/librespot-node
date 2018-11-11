#[macro_use]
extern crate neon;
extern crate librespot;
extern crate tokio_core;

#[macro_use]
extern crate log;
extern crate futures;
extern crate simple_logging;

use log::{ LevelFilter };

use neon::prelude::*;

mod lib {
    pub mod player;
}

use lib::player::SpotifyPlayer;

pub struct Spotify {
    player: SpotifyPlayer
}

declare_types! {
    pub class JsSpotify for Spotify {
        init(mut cx) {
            let username: Handle<JsString> = cx.argument::<JsString>(0)?;
            let password: Handle<JsString> = cx.argument::<JsString>(1)?;

            let player = SpotifyPlayer::new(username.value(), password.value());

            Ok(Spotify {
                player: player
            })
        }

        method play(mut cx) {
            let mut this = cx.this();
            let track_id: Handle<JsString> = cx.argument::<JsString>(0)?;

            {
                let guard = cx.lock();
                let mut spotify = this.borrow_mut(&guard);
                
                spotify.player.play(track_id.value());
            }

            Ok(cx.boolean(true).upcast())
        }

        method stop(mut cx) {
            let this = cx.this();   

            {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);
                
                spotify.player.stop();
            }

            Ok(cx.undefined().upcast())
        }

        method pause(mut cx) {
            let this = cx.this();   

            {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);
                
                spotify.player.pause();
            }

            Ok(cx.undefined().upcast())
        }

        method seek(mut cx) {
            let this = cx.this();
            let position_ms: Handle<JsNumber> = cx.argument::<JsNumber>(0)?; 

            {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);
                
                spotify.player.seek(position_ms.value() as u32);
            }

            Ok(cx.undefined().upcast())
        }

        method is_playing(mut cx) {
            let this = cx.this();   

            let is_playing = {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);
                
                spotify.player.is_playing()
            };

            Ok(cx.boolean(is_playing).upcast())
        }
    }
}

register_module!(mut cx, {
    simple_logging::log_to_stderr(LevelFilter::Info);

    cx.export_class::<JsSpotify>("Spotify")?;

    Ok(())
});
