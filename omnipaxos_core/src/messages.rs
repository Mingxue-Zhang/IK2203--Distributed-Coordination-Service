use serde::{Deserialize, Serialize};

use crate::{
    messages::{ballot_leader_election::BLEMessage, sequence_paxos::PaxosMessage},
    storage::{Entry, Snapshot},
    util::NodeId,
};

/// Internal component for log replication
pub mod sequence_paxos {
    use serde::{Serialize, Deserialize};
    use crate::{
        ballot_leader_election::Ballot,
        storage::{Entry, Snapshot, SnapshotType, StopSign},
        util::NodeId,
    };
    use std::fmt::Debug;

    /// Prepare message sent by a newly-elected leader to initiate the Prepare phase.
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct Prepare {
        /// The current round.
        pub n: Ballot,
        /// The decided index of this leader.
        pub decided_idx: u64,
        /// The latest round in which an entry was accepted.
        pub n_accepted: Ballot,
        /// The log length of this leader.
        pub accepted_idx: u64,
    }

    /// Promise message sent by a follower in response to a [`Prepare`] sent by the leader.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Promise<T, S>
    where
        T: Entry,
        S: Snapshot<T>,
    {
        /// The current round.
        pub n: Ballot,
        /// The latest round in which an entry was accepted.
        pub n_accepted: Ballot,
        /// The decided snapshot.
        pub decided_snapshot: Option<SnapshotType<T, S>>,
        /// The log suffix.
        pub suffix: Vec<T>,
        /// The decided index of this follower.
        pub decided_idx: u64,
        /// The log length of this follower.
        pub accepted_idx: u64,
        /// The StopSign accepted by this follower
        pub stopsign: Option<StopSign>,
    }

    /// AcceptSync message sent by the leader to synchronize the logs of all replicas in the prepare phase.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AcceptSync<T, S>
    where
        T: Entry,
        S: Snapshot<T>,
    {
        /// The current round.
        pub n: Ballot,
        /// The decided snapshot.
        pub decided_snapshot: Option<SnapshotType<T, S>>,
        /// The log suffix.
        pub suffix: Vec<T>,
        /// The index of the log where the entries from `sync_item` should be applied at or the compacted idx
        pub sync_idx: u64,
        /// The decided index
        pub decided_idx: u64,
        /// StopSign to be accepted
        pub stopsign: Option<StopSign>,
    }

    /// The first accept message sent. Only used by a pre-elected leader after reconfiguration.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct FirstAccept {
        /// The current round.
        pub n: Ballot,
    }

    /// Message with entries to be replicated and the latest decided index sent by the leader in the accept phase.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AcceptDecide<T>
    where
        T: Entry,
    {
        /// The current round.
        pub n: Ballot,
        /// The decided index.
        pub decided_idx: u64,
        /// Entries to be replicated.
        pub entries: Vec<T>,
    }

    /// Message sent by follower to leader when entries has been accepted.
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct Accepted {
        /// The current round.
        pub n: Ballot,
        /// The accepted index.
        pub accepted_idx: u64,
    }

    /// Message sent by leader to followers to decide up to a certain index in the log.
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct Decide {
        /// The current round.
        pub n: Ballot,
        /// The decided index.
        pub decided_idx: u64,
    }

    /// Message sent by leader to followers to accept a StopSign
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AcceptStopSign {
        /// The current round.
        pub n: Ballot,
        /// The decided index.
        pub ss: StopSign,
    }

    /// Message sent by followers to leader when accepted StopSign
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct AcceptedStopSign {
        /// The current round.
        pub n: Ballot,
    }

    /// Message sent by leader to decide a StopSign
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct DecideStopSign {
        /// The current round.
        pub n: Ballot,
    }

    /// Compaction Request
    #[allow(missing_docs)]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Compaction {
        Trim(u64),
        Snapshot(Option<u64>),
    }

    /// An enum for all the different message types.
    #[allow(missing_docs)]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum PaxosMsg<T, S>
    where
        T: Entry,
        S: Snapshot<T>,
    {
        /// Request a [`Prepare`] to be sent from the leader. Used for fail-recovery.
        PrepareReq,
        #[allow(missing_docs)]
        Prepare(Prepare),
        Promise(Promise<T, S>),
        AcceptSync(AcceptSync<T, S>),
        FirstAccept(FirstAccept),
        AcceptDecide(AcceptDecide<T>),
        Accepted(Accepted),
        Decide(Decide),
        /// Forward client proposals to the leader.
        ProposalForward(Vec<T>),
        Compaction(Compaction),
        AcceptStopSign(AcceptStopSign),
        AcceptedStopSign(AcceptedStopSign),
        DecideStopSign(DecideStopSign),
        ForwardStopSign(StopSign),
    }

    /// A struct for a Paxos message that also includes sender and receiver.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PaxosMessage<T, S>
    where
        T: Entry,
        S: Snapshot<T>,
    {
        /// Sender of `msg`.
        pub from: NodeId,
        /// Receiver of `msg`.
        pub to: NodeId,
        /// The message content.
        pub msg: PaxosMsg<T, S>,
    }
}

/// The different messages BLE uses to communicate with other replicas.
pub mod ballot_leader_election {
    use crate::{ballot_leader_election::Ballot, util::NodeId};
    use serde::{Serialize, Deserialize};

    /// An enum for all the different BLE message types.
    #[allow(missing_docs)]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum HeartbeatMsg {
        Request(HeartbeatRequest),
        Reply(HeartbeatReply),
    }

    /// Requests a reply from all the other replicas.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HeartbeatRequest {
        /// Number of the current round.
        pub round: u32,
    }

    /// Replies
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HeartbeatReply {
        /// Number of the current round.
        pub round: u32,
        /// Ballot of a replica.
        pub ballot: Ballot,
        /// States if the replica is a candidate to become a leader.
        pub quorum_connected: bool,
    }

    /// A struct for a Paxos message that also includes sender and receiver.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BLEMessage {
        /// Sender of `msg`.
        pub from: NodeId,
        /// Receiver of `msg`.
        pub to: NodeId,
        /// The message content.
        pub msg: HeartbeatMsg,
    }
}

#[allow(missing_docs)]
/// Message in OmniPaxos. Can be either a `SequencePaxos` message (for log replication) or `BLE` message (for leader election)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message<T, S>
where
    T: Entry,
    S: Snapshot<T>,
{
    SequencePaxos(PaxosMessage<T, S>),
    /// Should be Serializable and Deserializable.
    BLE(BLEMessage),
}

impl<T, S> Message<T, S>
where
    T: Entry,
    S: Snapshot<T>,
{
    /// Get the sender id of the message
    pub fn get_sender(&self) -> NodeId {
        match self {
            Message::SequencePaxos(p) => p.from,
            Message::BLE(b) => b.from,
        }
    }

    /// Get the receiver id of the message
    pub fn get_receiver(&self) -> NodeId {
        match self {
            Message::SequencePaxos(p) => p.to,
            Message::BLE(b) => b.to,
        }
    }
}
