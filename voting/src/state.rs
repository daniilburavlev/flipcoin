use std::collections::HashSet;

use num_bigint::BigUint;

use crate::vote::Vote;

pub(crate) struct VotingState {
    current: BigUint,
    threshold: BigUint,
    voted: HashSet<String>,
}

impl VotingState {
    pub(crate) fn new(total: BigUint) -> Self {
        let threshold = total * BigUint::from(2u64) / BigUint::from(3u64);
        Self {
            current: BigUint::default(),
            threshold,
            voted: HashSet::new(),
        }
    }

    pub(crate) fn vote(&mut self, vote: Vote) -> bool {
        if self.voted.contains(&vote.voter) {
            return false;
        }
        self.voted.insert(vote.voter);
        self.current += vote.stake;
        self.current >= self.threshold
    }
}
