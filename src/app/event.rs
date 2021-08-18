use std::time::{Duration, Instant};

pub enum EventResult {
    Quit,
    Redraw,
    Continue,
}

enum MultiKeyState {
    /// Currently not waiting on any next input.
    Idle,

    /// Waiting for the next input, with a given trigger [`Instant`].
    Waiting {
        /// When it was triggered.
        trigger_instant: Instant,

        /// What part of the pattern it is at.
        checked_index: usize,
    },
}

/// The possible outcomes of calling [`MultiKey::input`].
pub enum MultiKeyResult {
    /// Returned when a character was *accepted*, but has not completed the sequence required.
    Accepted,

    /// Returned when a character is accepted and completes the sequence.
    Completed,

    /// Returned if a character breaks the sequence or if it has already timed out.
    Rejected,
}

/// A struct useful for managing multi-key keybinds.
pub struct MultiKey {
    state: MultiKeyState,
    pattern: Vec<char>,
    timeout: Duration,
}

impl MultiKey {
    /// Creates a new [`MultiKey`] with a given pattern and timeout.
    pub fn register(pattern: Vec<char>, timeout: Duration) -> Self {
        Self {
            state: MultiKeyState::Idle,
            pattern,
            timeout,
        }
    }

    /// Resets to an idle state.
    fn reset(&mut self) {
        self.state = MultiKeyState::Idle;
    }

    /// Handles a char input and returns the current status of the [`MultiKey`] after, which is one of:
    /// - Accepting the char and moving to the next state
    /// - Completing the multi-key pattern
    /// - Rejecting it
    ///
    /// Note that if a [`MultiKey`] only "times out" upon calling this - if it has timed out, it will first reset
    /// before trying to check the char.
    pub fn input(&mut self, c: char) -> MultiKeyResult {
        match &mut self.state {
            MultiKeyState::Idle => {
                if let Some(first) = self.pattern.first() {
                    if *first == c {
                        self.state = MultiKeyState::Waiting {
                            trigger_instant: Instant::now(),
                            checked_index: 0,
                        };

                        return MultiKeyResult::Accepted;
                    }
                }

                MultiKeyResult::Rejected
            }
            MultiKeyState::Waiting {
                trigger_instant,
                checked_index,
            } => {
                if trigger_instant.elapsed() > self.timeout {
                    // Just reset and recursively call (putting it into Idle).
                    self.reset();
                    self.input(c)
                } else if let Some(next) = self.pattern.get(*checked_index + 1) {
                    if *next == c {
                        *checked_index += 1;

                        if *checked_index == self.pattern.len() - 1 {
                            self.reset();
                            MultiKeyResult::Completed
                        } else {
                            MultiKeyResult::Accepted
                        }
                    } else {
                        self.reset();
                        MultiKeyResult::Rejected
                    }
                } else {
                    self.reset();
                    MultiKeyResult::Rejected
                }
            }
        }
    }
}
