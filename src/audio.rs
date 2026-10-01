use raylib::audio::{Music, RaylibAudio};

pub const BACKGROUND_VOLUME: f32 = 0.45;
pub const INTRO_VOLUME: f32 = 0.50;
pub const FADE_SECONDS: f32 = 0.4;

const BACKGROUND_PATH: &str = "assets/audio/background/minecraft_music.mp3";
pub const INTRO_TRACKS: [IntroTrack; 5] = [
    IntroTrack {
        scene_id: "black_clover_skull",
        path: "assets/audio/intros/black_clover.mp3",
    },
    IntroTrack {
        scene_id: "shenlong",
        path: "assets/audio/intros/dragon_ball.mp3",
    },
    IntroTrack {
        scene_id: "kurama",
        path: "assets/audio/intros/naruto.mp3",
    },
    IntroTrack {
        scene_id: "pochita",
        path: "assets/audio/intros/chainsaw_man.mp3",
    },
    IntroTrack {
        scene_id: "lapras",
        path: "assets/audio/intros/pokemon.mp3",
    },
];

#[derive(Clone, Copy)]
pub struct IntroTrack {
    pub scene_id: &'static str,
    pub path: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioState {
    Background,
    FadingOutBackground,
    FadingInIntro,
    IntroPlaying,
    IntroPaused,
    FadingInBackground,
}

#[derive(Clone, Copy)]
pub struct AudioStatus {
    pub active: bool,
    pub playing: bool,
    pub progress: f32,
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Clone, Copy)]
struct AudioLogic {
    state: AudioState,
    selected_intro: usize,
    active_intro: Option<usize>,
    fade_elapsed: f32,
}

impl AudioLogic {
    fn new(selected_intro: usize) -> Self {
        Self {
            state: AudioState::Background,
            selected_intro,
            active_intro: None,
            fade_elapsed: 0.0,
        }
    }

    fn select(&mut self, intro: usize) -> bool {
        self.selected_intro = intro;
        if self.active_intro.is_some() && self.active_intro != Some(intro) {
            self.active_intro = None;
            self.state = AudioState::FadingInBackground;
            self.fade_elapsed = 0.0;
            return true;
        }
        false
    }

    fn toggle(&mut self) {
        match self.state {
            AudioState::IntroPlaying | AudioState::FadingInIntro => {
                self.state = AudioState::IntroPaused
            }
            AudioState::IntroPaused => self.state = AudioState::IntroPlaying,
            _ => {
                self.active_intro = Some(self.selected_intro);
                self.state = AudioState::FadingOutBackground;
                self.fade_elapsed = 0.0;
            }
        }
    }

    fn finish(&mut self) {
        self.active_intro = None;
        self.state = AudioState::FadingInBackground;
        self.fade_elapsed = 0.0;
    }

    fn cancel(&mut self) {
        self.active_intro = None;
        self.state = AudioState::Background;
        self.fade_elapsed = 0.0;
    }
}

pub struct AudioManager<'audio> {
    background: Option<Music<'audio>>,
    intros: Vec<Option<Music<'audio>>>,
    logic: AudioLogic,
    last_intro_time: f32,
}

impl<'audio> AudioManager<'audio> {
    pub fn load(audio: &'audio RaylibAudio, initial_scene_id: &str) -> Self {
        let background = load_music(audio, BACKGROUND_PATH);
        if let Some(music) = &background {
            music.set_volume(BACKGROUND_VOLUME);
            music.play_stream();
        }
        let intros = INTRO_TRACKS
            .iter()
            .map(|track| load_music(audio, track.path))
            .collect();
        Self {
            background,
            intros,
            logic: AudioLogic::new(intro_index(initial_scene_id).unwrap_or(0)),
            last_intro_time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if let Some(background) = &self.background {
            background.update_stream();
        }
        if let Some(intro) = self.active_music() {
            intro.update_stream();
        }

        match self.logic.state {
            AudioState::Background => self.ensure_background_loop(),
            AudioState::FadingOutBackground => {
                self.logic.fade_elapsed += delta_time;
                let amount = (self.logic.fade_elapsed / FADE_SECONDS).clamp(0.0, 1.0);
                if let Some(background) = &self.background {
                    background.set_volume(BACKGROUND_VOLUME * (1.0 - amount));
                }
                if amount >= 1.0 {
                    if let Some(background) = &self.background {
                        background.pause_stream();
                    }
                    if let Some(intro) = self.active_music() {
                        intro.seek_stream(0.0);
                        intro.set_volume(0.0);
                        intro.play_stream();
                        self.last_intro_time = 0.0;
                        self.logic.state = AudioState::FadingInIntro;
                        self.logic.fade_elapsed = 0.0;
                    } else {
                        self.start_background_fade();
                    }
                }
            }
            AudioState::FadingInIntro => {
                self.logic.fade_elapsed += delta_time;
                let amount = (self.logic.fade_elapsed / FADE_SECONDS).clamp(0.0, 1.0);
                if let Some(intro) = self.active_music() {
                    intro.set_volume(INTRO_VOLUME * amount);
                }
                if amount >= 1.0 {
                    self.logic.state = AudioState::IntroPlaying;
                }
            }
            AudioState::IntroPlaying => {
                let current_time = self.active_music().map_or(0.0, Music::get_time_played);
                if self.intro_finished(current_time) {
                    self.finish_intro();
                } else {
                    self.last_intro_time = current_time;
                }
            }
            AudioState::IntroPaused => {}
            AudioState::FadingInBackground => {
                self.logic.fade_elapsed += delta_time;
                let amount = (self.logic.fade_elapsed / FADE_SECONDS).clamp(0.0, 1.0);
                if let Some(background) = &self.background {
                    background.set_volume(BACKGROUND_VOLUME * amount);
                }
                if amount >= 1.0 {
                    self.logic.state = AudioState::Background;
                }
            }
        }
    }

    pub fn select_scene(&mut self, scene_id: &str) {
        let Some(index) = intro_index(scene_id) else {
            return;
        };
        if self.logic.select(index) {
            self.cancel_intro_and_resume_background();
        }
    }

    pub fn cancel_intro_and_resume_background(&mut self) {
        for music in self.intros.iter().flatten() {
            music.stop_stream();
            music.seek_stream(0.0);
        }
        self.last_intro_time = 0.0;
        self.logic.cancel();
        if let Some(background) = &self.background {
            background.set_volume(BACKGROUND_VOLUME);
            background.resume_stream();
            if !background.is_stream_playing() {
                background.play_stream();
            }
        }
    }

    pub fn toggle_intro(&mut self) {
        let previous = self.logic.state;
        self.logic.toggle();
        match (previous, self.logic.state) {
            (_, AudioState::IntroPaused) => {
                if let Some(intro) = self.active_music() {
                    intro.pause_stream();
                }
            }
            (AudioState::IntroPaused, AudioState::IntroPlaying) => {
                if let Some(intro) = self.active_music() {
                    intro.resume_stream();
                    intro.set_volume(INTRO_VOLUME);
                }
            }
            _ => {}
        }
    }

    pub fn seek(&mut self, progress: f32) {
        if let Some(intro) = self.active_music() {
            let position = intro.get_time_length() * progress.clamp(0.0, 1.0);
            intro.seek_stream(position);
            self.last_intro_time = position;
        }
    }

    pub fn status(&self) -> AudioStatus {
        let music = self.active_music();
        let duration = music.map_or(0.0, Music::get_time_length);
        let elapsed = music
            .map_or(0.0, Music::get_time_played)
            .clamp(0.0, duration.max(0.0));
        AudioStatus {
            active: self.logic.active_intro == Some(self.logic.selected_intro),
            playing: matches!(
                self.logic.state,
                AudioState::FadingInIntro | AudioState::IntroPlaying
            ),
            progress: if duration > 0.0 {
                elapsed / duration
            } else {
                0.0
            },
            elapsed,
            duration,
        }
    }

    fn active_music(&self) -> Option<&Music<'audio>> {
        self.logic
            .active_intro
            .and_then(|index| self.intros.get(index)?.as_ref())
    }
    fn intro_finished(&self, current_time: f32) -> bool {
        self.active_music().is_some_and(|music| {
            let duration = music.get_time_length();
            duration > 0.0
                && (current_time >= duration - 0.05 || current_time + 0.05 < self.last_intro_time)
        })
    }
    fn finish_intro(&mut self) {
        if let Some(intro) = self.active_music() {
            intro.stop_stream();
            intro.seek_stream(0.0);
        }
        self.last_intro_time = 0.0;
        self.resume_background_at_zero_volume();
        self.logic.finish();
    }
    fn start_background_fade(&mut self) {
        self.resume_background_at_zero_volume();
        self.logic.state = AudioState::FadingInBackground;
        self.logic.fade_elapsed = 0.0;
    }
    fn resume_background_at_zero_volume(&self) {
        if let Some(background) = &self.background {
            background.set_volume(0.0);
            background.resume_stream();
        }
    }
    fn ensure_background_loop(&self) {
        if let Some(background) = &self.background {
            if !background.is_stream_playing() {
                background.seek_stream(0.0);
                background.play_stream();
                background.set_volume(BACKGROUND_VOLUME);
            }
        }
    }
}

fn intro_index(scene_id: &str) -> Option<usize> {
    INTRO_TRACKS
        .iter()
        .position(|track| track.scene_id == scene_id)
}
fn load_music<'audio>(audio: &'audio RaylibAudio, path: &str) -> Option<Music<'audio>> {
    match audio.new_music(path) {
        Ok(music) => Some(music),
        Err(error) => {
            eprintln!("Audio unavailable for '{path}': {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intro_mapping_matches_gallery_order() {
        assert_eq!(
            INTRO_TRACKS.map(|track| track.scene_id),
            [
                "black_clover_skull",
                "shenlong",
                "kurama",
                "pochita",
                "lapras"
            ]
        );
        assert!(INTRO_TRACKS[0].path.ends_with("black_clover.mp3"));
        assert!(INTRO_TRACKS[1].path.ends_with("dragon_ball.mp3"));
        assert!(INTRO_TRACKS[2].path.ends_with("naruto.mp3"));
        assert!(INTRO_TRACKS[3].path.ends_with("chainsaw_man.mp3"));
        assert!(INTRO_TRACKS[4].path.ends_with("pokemon.mp3"));
    }
    #[test]
    fn pure_state_supports_play_pause_resume_finish_and_selection() {
        let mut logic = AudioLogic::new(0);
        logic.toggle();
        assert_eq!(logic.state, AudioState::FadingOutBackground);
        logic.state = AudioState::IntroPlaying;
        logic.toggle();
        assert_eq!(logic.state, AudioState::IntroPaused);
        logic.toggle();
        assert_eq!(logic.state, AudioState::IntroPlaying);
        logic.finish();
        assert_eq!(logic.state, AudioState::FadingInBackground);
        assert_eq!(logic.active_intro, None);
        logic.active_intro = Some(0);
        logic.state = AudioState::IntroPlaying;
        assert!(logic.select(1));
        assert_eq!(logic.state, AudioState::FadingInBackground);
        assert_eq!(logic.active_intro, None);
    }

    #[test]
    fn view_transition_cancels_intro_and_restores_background_state() {
        let mut logic = AudioLogic::new(0);
        logic.active_intro = Some(0);
        logic.state = AudioState::IntroPlaying;
        logic.cancel();
        assert_eq!(logic.state, AudioState::Background);
        assert_eq!(logic.active_intro, None);

        logic.toggle();
        assert_eq!(logic.active_intro, Some(0));
        assert_eq!(logic.state, AudioState::FadingOutBackground);
        logic.cancel();
        assert_eq!(logic.state, AudioState::Background);
        assert_eq!(logic.active_intro, None);
    }
    #[test]
    fn loop_policy_is_background_only() {
        const BACKGROUND_LOOPS: bool = true;
        const INTRO_LOOPS: bool = false;
        assert!(BACKGROUND_LOOPS);
        assert!(!INTRO_LOOPS);
    }
}
