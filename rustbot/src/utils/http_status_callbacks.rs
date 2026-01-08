use rsbwapi::Game;
use std::sync::{Arc, Mutex};

use super::game_state::GameState;

pub type StatusCallback = Box<dyn FnOnce(&Game, &GameState) + Send>;

pub struct HttpStatusCallbacks {
  callbacks: Vec<StatusCallback>,
}

impl HttpStatusCallbacks {
  pub fn new() -> Self {
    Self {
      callbacks: Vec::new(),
    }
  }

  pub fn add_callback(&mut self, callback: StatusCallback) {
    self.callbacks.push(callback);
  }

  pub fn process_all(&mut self, game: &Game, state: &GameState) {
    let callbacks = std::mem::take(&mut self.callbacks);
    for callback in callbacks {
      callback(game, state);
    }
  }

  pub fn has_pending(&self) -> bool {
    !self.callbacks.is_empty()
  }
}

pub type SharedHttpStatusCallbacks = Arc<Mutex<HttpStatusCallbacks>>;
