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
    pub mod token;
}

use lib::player::SpotifyPlayer;
use lib::token::{ AccessToken, JsAccessToken };

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

            Ok(cx.undefined().upcast())
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

        method isPlaying(mut cx) {
            let this = cx.this();

            let is_playing = {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);

                spotify.player.is_playing()
            };

            Ok(cx.boolean(is_playing).upcast())
        }

        method getToken(mut cx) {
            let this = cx.this();
            let ctor = JsAccessToken::constructor(&mut cx)?;

            let client_id: Handle<JsString> = cx.argument::<JsString>(0)?;
            let scopes: Handle<JsString> = cx.argument::<JsString>(1)?;
            let cb: Handle<JsFunction> = cx.argument::<JsFunction>(2)?;

            let mut token: Option<AccessToken> = None;

            {
                let guard = cx.lock();
                let spotify = this.borrow(&guard);

                spotify.player.get_token(client_id.value(), scopes.value(), |tok| {
                    match tok {
                        Some(t) => {
                            token = Some(AccessToken {
                                token: t.access_token,
                                scope: t.scope,
                                expires_in: t.expires_in
                            });
                        },
                        None => {
                            token = None;
                        }
                    };
                });
            }

            match token {
                Some(t) => {
                    let scopes = JsArray::new(&mut cx, t.scope.len() as u32);
                    for (i, scope) in t.scope.iter().enumerate() {
                        let val = cx.string(scope);
                        let _ = scopes.set(&mut cx, i as u32, val);
                    }

                    let args: Vec<Handle<JsValue>> = vec![
                        cx.string(t.token).upcast(),
                        scopes.upcast(),
                        cx.number(t.expires_in).upcast()
                    ];

                    let access_token_instance = ctor.construct(&mut cx, args);

                    let cb_args: Vec<Handle<JsValue>> = vec![
                        access_token_instance.unwrap().upcast(),
                    ];

                    let _ = cb.call(&mut cx, JsNull::new(), cb_args);
                },
                None => {
                    let _ = cb.call(&mut cx, JsNull::new(), vec![ JsUndefined::new() ]);
                }
            }

            Ok(cx.undefined().upcast())
        }
    }
}

register_module!(mut cx, {
    simple_logging::log_to_stderr(LevelFilter::Info);

    cx.export_class::<JsSpotify>("Spotify")?;

    Ok(())
});
