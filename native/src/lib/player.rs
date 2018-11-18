use tokio_core::reactor::{ Core, Remote };

use futures::{ Future, Stream, future };
use futures::sync::oneshot;
use std::{ thread };
use std::sync::{ Mutex, Arc };

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;

use librespot::core::spotify_id::SpotifyId;
use librespot::core::keymaster;
use librespot::core::keymaster::Token;
use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;
use librespot::playback::player::PlayerEvent;

pub struct PlayerState {
    is_playing: bool,
}

pub struct SpotifyPlayer {
    remote: Remote,
    player: Player,
    session: Session,
    state: Arc<Mutex<PlayerState>>
}

impl SpotifyPlayer {
    pub fn new(username: String, password: String) -> SpotifyPlayer {
        let (session_tx, session_rx) = oneshot::channel();
        let (remote_tx, remote_rx) = oneshot::channel();

        let credentials = Credentials::with_password(username, password);

        thread::spawn(move || {
            let mut core = Core::new().unwrap();

            let session_config = SessionConfig::default();
            let handle = core.handle();

            let _ = remote_tx.send(handle.remote().clone());

            let session = core.run(Session::connect(
                session_config,
                credentials,
                None,
                handle)).unwrap();

            let _ = session_tx.send(session);

            core.run(future::empty::<(), ()>()).unwrap();
        });

        let remote = remote_rx.wait().unwrap();
        let session = session_rx.wait().unwrap();

        let backend = audio_backend::find(None).unwrap();
        let player_config = PlayerConfig::default();
        let (player, rx) = Player::new(player_config, session.clone(), None, move || (backend)(None));

        let state = Arc::new(Mutex::new(PlayerState {
            is_playing: false
        }));

        let local_state = state.clone();

        remote.spawn(move |_| {
            rx.for_each(move |res| {
                debug!("PlayerEvent: {:?}", res);

                let mut state = local_state.lock().unwrap();

                match res {
                    PlayerEvent::Started { .. } => state.is_playing = true,
                    PlayerEvent::Changed { .. } => debug!("Player state changed"),
                    PlayerEvent::Stopped { .. } => state.is_playing = false,
                }

                Ok(())
            })
        });

        SpotifyPlayer {
            remote: remote,
            player: player,
            session: session,
            state: state,
        }
    }

    pub fn play(&mut self, track_id: String) {
        let track = SpotifyId::from_base62(&track_id).unwrap();

        info!("Track: {:?}", track);

        let end_of_track = self.player.load(track, true, 0);
        let local_state = self.state.clone();

        self.remote.spawn(move |_| {
            end_of_track.then(move |_result| {
                let mut state = local_state.lock().unwrap();
                state.is_playing = false;

                Ok(())
            })
        });
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn seek(&self, position_ms: u32) {
        self.player.seek(position_ms);
    }

    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().is_playing
    }

    pub fn get_token<F>(&self, client_id: String, scopes: String, cb: F)
        where F: FnOnce(Token) {

        let local_session = self.session.clone();
        let (token_tx, token_rx) = oneshot::channel();

        self.remote.spawn(move |_| {
            keymaster::get_token(&local_session, &client_id, &scopes).then(move |res| {
                let _ = token_tx.send(res);
                Ok(())
            })
        });

        match token_rx.wait().unwrap() {
            Ok(r) => {
                cb(r);
            },
            Err(e) => {
                error!("Cannot get token {:?}", e);
            }
        };
    }
}