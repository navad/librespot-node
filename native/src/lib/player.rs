use tokio_core::reactor::{ Core, Remote };

use futures::{ Future, future };
use futures::sync::oneshot;
use std::{ thread };
use std::sync::{ Mutex, Arc };

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;

use librespot::core::spotify_id::SpotifyId;
use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

pub struct PlayerState {
    is_playing: bool,
}

pub struct SpotifyPlayer {
    remote: Remote,
    player: Player,
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
        let (player, _) = Player::new(player_config, session.clone(), None, move || (backend)(None));

        SpotifyPlayer {
            remote: remote,
            player: player,
            state: Arc::new(Mutex::new(PlayerState {
                is_playing: false
            }))
        }
    }

    pub fn play(&mut self, track_id: String) {
        let track = SpotifyId::from_base62(&track_id).unwrap();

        info!("Track: {:?}", track);

        let end_of_track = self.player.load(track, true, 0);
        let local_state = self.state.clone();

        self.state.lock().unwrap().is_playing = true;        

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
}