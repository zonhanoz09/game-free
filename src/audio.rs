use bevy::prelude::*;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEffect {
    Click,
    Slash,
    Arrow,
    Magic,
    Dagger,
    Heal,
    Hit,
    Shield,
    Ultimate,
    Victory,
    Defeat,
}

impl SoundEffect {
    pub fn file_path(&self) -> &'static str {
        match self {
            SoundEffect::Click => "assets/audio/ui_click.wav",
            SoundEffect::Slash => "assets/audio/attack_slash.wav",
            SoundEffect::Arrow => "assets/audio/attack_arrow.wav",
            SoundEffect::Magic => "assets/audio/attack_magic.wav",
            SoundEffect::Dagger => "assets/audio/attack_dagger.wav",
            SoundEffect::Heal => "assets/audio/spell_heal.wav",
            SoundEffect::Hit => "assets/audio/hit_impact.wav",
            SoundEffect::Shield => "assets/audio/shield_block.wav",
            SoundEffect::Ultimate => "assets/audio/ultimate_cast.wav",
            SoundEffect::Victory => "assets/audio/victory.wav",
            SoundEffect::Defeat => "assets/audio/defeat.wav",
        }
    }
}

#[derive(Event)]
pub struct PlaySoundEvent(pub SoundEffect);

#[derive(Resource)]
pub struct SoundManager {
    sender: Sender<SoundEffect>,
    pub muted: bool,
}

impl Default for SoundManager {
    fn default() -> Self {
        let (sender, receiver): (Sender<SoundEffect>, Receiver<SoundEffect>) = channel();

        thread::spawn(move || {
            while let Ok(sound) = receiver.recv() {
                let path = sound.file_path();
                let _ = std::process::Command::new("aplay")
                    .arg("-q")
                    .arg(path)
                    .spawn()
                    .or_else(|_| std::process::Command::new("pw-play").arg(path).spawn());
            }
        });

        Self {
            sender,
            muted: false,
        }
    }
}

impl SoundManager {
    pub fn play(&self, sound: SoundEffect) {
        if !self.muted {
            let _ = self.sender.send(sound);
        }
    }
}

pub fn sound_event_listener(mut events: EventReader<PlaySoundEvent>, sound_mgr: Res<SoundManager>) {
    for ev in events.read() {
        info!("[AUDIO] Sfx played: {:?}", ev.0);
        sound_mgr.play(ev.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_file_paths() {
        let sounds = [
            SoundEffect::Click,
            SoundEffect::Slash,
            SoundEffect::Arrow,
            SoundEffect::Magic,
            SoundEffect::Dagger,
            SoundEffect::Heal,
            SoundEffect::Hit,
            SoundEffect::Shield,
            SoundEffect::Ultimate,
            SoundEffect::Victory,
            SoundEffect::Defeat,
        ];
        for s in sounds {
            assert!(s.file_path().ends_with(".wav"));
        }
    }
}
