use std::collections::HashMap;

use futures::lock::Mutex;
use num_bigint::BigUint;

use crate::state::VotingState;

pub(crate) mod state;
pub(crate) mod vote;

pub use crate::vote::Vote;

pub struct Voting {
    state: Mutex<HashMap<u64, VotingState>>,
}

impl Default for Voting {
    fn default() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl Voting {
    pub async fn start(&self, height: u64, total: BigUint) {
        let mut lock = self.state.lock().await;
        lock.entry(height).or_insert(VotingState::new(total));
    }

    pub async fn vote(&self, vote: Vote) -> bool {
        let mut lock = self.state.lock().await;
        if let Some(state) = lock.get_mut(&vote.block_height) {
            state.vote(vote)
        } else {
            false
        }
    }
}
