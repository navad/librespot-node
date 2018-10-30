use tokio_core::reactor::{ Core, Remote };

use futures::{ Future, future };
use futures::sync::oneshot;
use std::{ thread };

use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;

use librespot::core::spotify_id::SpotifyId;
use librespot::playback::config::PlayerConfig;
use librespot::playback::audio_backend;
use librespot::playback::player::Player;

pub struct SpotifyPlayer {
    remote: Remote,
    player: Player
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

            info!("Getting session...");

            let session = core.run(Session::connect(
                session_config,
                credentials,
                None,
                handle)).unwrap();

            let _ = session_tx.send(session);

            info!("Starting main loop");

            core.run(future::empty::<(), ()>()).unwrap();
        });

        let remote = remote_rx.wait().unwrap();
        let session = session_rx.wait().unwrap();

        let backend = audio_backend::find(None).unwrap();
        let player_config = PlayerConfig::default();
        let (player, _) = Player::new(player_config, session.clone(), None, move || (backend)(None));

        SpotifyPlayer {
            remote: remote,
            player: player
        }
    }

    pub fn play(&self, track_id: String) {
        let track = SpotifyId::from_base62(&track_id).unwrap();
        info!("Track: {:?}", track);

        let end_of_track = self.player.load(track, true, 0);

        self.remote.spawn(move |_| {
            info!("In remote thread");

            end_of_track.then(move |result| {
                info!("Track playback ended {:?}", result);
                Ok(())
            })
        });
    }

    pub fn stop(&self) {
        self.player.stop();
    }
}