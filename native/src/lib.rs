#[macro_use]
extern crate neon;
extern crate librespot;
extern crate tokio_core;

#[macro_use]
extern crate log;
extern crate futures;
extern crate simple_logging;

use log::{ LevelFilter };

use neon::context::Context;
use neon::prelude::*;

mod lib {
    pub mod player;
}

use lib::player::SpotifyPlayer;

pub struct Spotify {
    player: SpotifyPlayer
}

pub struct AccessToken {
    token: String,
    scope: Vec<String>,
    expires_in: u32,
}

declare_types! {
    pub class JsAccessToken for AccessToken {
        init(mut cx) {
            let token: Handle<JsString> = cx.argument::<JsString>(0)?;
            let expires_in: Handle<JsNumber> = cx.argument::<JsNumber>(1)?;

            Ok(AccessToken {
                token: token.value(),
                scope: vec![ ],
                expires_in: expires_in.value() as u32,
            })
        }

        method getToken(mut cx) {
            let this = cx.this();
            let val = {
                let guard = cx.lock();
                let instance = this.borrow(&guard);

                instance.token.clone()
            };

            Ok(cx.string(val).upcast())
        }

        method getExpiry(mut cx) {
            let this = cx.this();
            let val = {
                let guard = cx.lock();
                let instance = this.borrow(&guard);

                instance.expires_in.clone()
            };

            Ok(cx.number(val).upcast())
        }

    }
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

        method get_token(mut cx) {
            let this = cx.this();
            let ctor = JsAccessToken::constructor(&mut cx)?;

            let client_id: Handle<JsString> = cx.argument::<JsString>(0)?;
            let scopes: Handle<JsString> = cx.argument::<JsString>(1)?;
            let cb: Handle<JsFunction> = cx.argument::<JsFunction>(2)?;

            let mut token: Option<AccessToken> = None;

            {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);

                spotify.player.get_token(
                    client_id.value(),
                    scopes.value(),
                    |tok| {
                        token = Some(AccessToken {
                            token: tok.access_token,
                            scope: vec![ ],
                            expires_in: tok.expires_in
                        });
                    });
            }

            let tok = token.unwrap();

            let args: Vec<Handle<JsValue>> = vec![
                cx.string(tok.token).upcast(),
                cx.number(tok.expires_in).upcast()
            ];

            let access_token_instance = ctor.construct(&mut cx, args);

            let cb_args: Vec<Handle<JsValue>> = vec![
                access_token_instance.unwrap().upcast(),
            ];

            let _ = cb.call(&mut cx, JsNull::new(), cb_args);

            Ok(cx.undefined().upcast())
        }
    }
}

register_module!(mut cx, {
    simple_logging::log_to_stderr(LevelFilter::Info);

    cx.export_class::<JsSpotify>("Spotify")?;

    Ok(())
});
