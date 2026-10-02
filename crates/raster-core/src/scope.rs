//! The planner and fonts-aware adapter share the same typed coordinate state.
use crate::{Command, Frame, MAX_COORDINATE, MAX_SCOPES, Profile, Rect, Result, reserved};

#[derive(Clone, Copy)]
pub(crate) enum Scope {
    Clip(Rect),
    Fixed { clip: Rect, offset: (f32, f32) },
}

pub struct CoordinateState {
    viewport: Rect,
    caller: Rect,
    clip: Rect,
    offset: (f32, f32),
    fixed_offset: (f32, f32),
    scopes: Vec<Scope>,
}

impl CoordinateState {
    pub fn new(frame: Frame) -> Result<Self> {
        Self::new_for_profile(Profile::Probe, frame)
    }

    pub fn new_for_profile(profile: Profile, frame: Frame) -> Result<Self> {
        profile.validate_viewport(frame.width, frame.height)?;
        if frame.clear > 0x00ff_ffff {
            return Err("clear color must be packed RGB".into());
        }
        frame.caller_clip.validate()?;
        for value in [
            frame.document_offset.0,
            frame.document_offset.1,
            frame.viewport_offset.0,
            frame.viewport_offset.1,
        ] {
            if !value.is_finite() || value.abs() > MAX_COORDINATE {
                return Err("invalid offset".into());
            }
        }
        let viewport = Rect::new(0.0, 0.0, frame.width as f32, frame.height as f32);
        let caller = frame.caller_clip.normalized_clip(viewport);
        Ok(Self {
            viewport,
            caller,
            clip: caller,
            offset: frame.document_offset,
            fixed_offset: frame.viewport_offset,
            scopes: reserved(MAX_SCOPES)?,
        })
    }

    pub fn clip(&self) -> Rect {
        self.clip
    }
    pub fn offset(&self) -> (f32, f32) {
        self.offset
    }

    /// Returns true only for a handled scope command. Drawing commands do not
    /// change state and must be validated by their own preparation path.
    pub fn apply(&mut self, command: &Command) -> Result<bool> {
        match *command {
            Command::PushClip(rect) => {
                rect.validate()?;
                if self.scopes.len() == MAX_SCOPES {
                    return Err("scope budget".into());
                }
                self.scopes.push(Scope::Clip(self.clip));
                // Preserve the old f32 add/intersection/normalization order.
                self.clip = self
                    .clip
                    .intersect(rect.translated(self.offset))
                    .normalized_clip(self.viewport);
            }
            Command::PopClip => match self.scopes.pop() {
                Some(Scope::Clip(old)) => self.clip = old,
                _ => return Err("mismatched clip scope".into()),
            },
            Command::PushFixed => {
                if self.scopes.len() == MAX_SCOPES {
                    return Err("scope budget".into());
                }
                self.scopes.push(Scope::Fixed {
                    clip: self.clip,
                    offset: self.offset,
                });
                self.clip = self.caller;
                self.offset = self.fixed_offset;
            }
            Command::PopFixed => match self.scopes.pop() {
                Some(Scope::Fixed { clip, offset }) => {
                    self.clip = clip;
                    self.offset = offset;
                }
                _ => return Err("mismatched fixed scope".into()),
            },
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub fn finish(&self) -> Result<()> {
        if self.scopes.is_empty() {
            Ok(())
        } else {
            Err("unclosed scope".into())
        }
    }
}
