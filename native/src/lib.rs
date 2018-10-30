#[macro_use]
extern crate neon;
extern crate librespot;
extern crate tokio_core;

#[macro_use]
extern crate log;
extern crate futures;
extern crate simple_logger;
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
            let this = cx.this();   

            {
                let guard = cx.lock();
                let tester = this.borrow(&guard);
                
                tester.player.play("5Fhn1eNY4Eexndp9DWHZnn".to_string());
            }

            Ok(cx.boolean(true).upcast())
        }

        method stop(mut cx) {
            let this = cx.this();   

            {
                let guard = cx.lock();
                let tester = this.borrow(&guard);
                
                tester.player.stop();
            }

            Ok(cx.boolean(true).upcast())
        }
    }
}

register_module!(mut cx, {
    simple_logging::log_to_stderr(LevelFilter::Info);

    cx.export_class::<JsSpotify>("Spotify")?;

    Ok(())
});
