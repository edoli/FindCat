//! Unicode-safe dialogue timing, independent of rendering and input devices.
#[derive(Clone, Debug, Default)]
pub struct Dialogue {
    pub text: String,
    end: usize,
    budget: f32,
}

impl Dialogue {
    pub fn set_text(&mut self, text: &str) {
        if self.text != text {
            self.text = text.into();
            self.end = 0;
            self.budget = 0.;
        }
    }
    pub fn typing(&self) -> bool {
        self.end < self.text.len()
    }
    pub fn visible(&self) -> &str {
        &self.text[..self.end]
    }
    pub fn remaining(&self) -> &str {
        &self.text[self.end..]
    }
    pub fn finish(&mut self) {
        self.end = self.text.len();
        self.budget = 0.;
    }
    pub fn tick(&mut self, seconds: f32) {
        self.budget += seconds.max(0.);
        while let Some(ch) = self.remaining().chars().next() {
            let delay = match ch {
                '.' | '!' | '?' | '…' | '。' => 0.16,
                ',' | '，' | '\n' => 0.09,
                _ => 1. / 32.,
            };
            if self.budget < delay {
                break;
            }
            self.budget -= delay;
            self.end += ch.len_utf8();
        }
        if !self.typing() {
            self.budget = 0.;
        }
    }
}
