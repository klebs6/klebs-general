// ---------------- [ File: src/errors.rs ]
#![allow(unused)]
crate::ix!();

error_tree!{

    // Represents all possible errors that might arise while
    // constructing or executing the network.
    #[derive(Clone,PartialEq)]
    pub enum NetworkError {
        DowncastError(unsafe_erased::DowncastError),
        InvalidPinAssignment,

        /// we use this one to help during hydro2-network-wire-derive
        PortTryFromNull,

        TrySendError(TrySendError<usize>),

        Timeout {
            message: String,
        },
        /// A configuration was invalid, e.g., referencing
        /// missing nodes, edges, or incompatible buffer types.
        InvalidConfiguration {
            // Additional details about the invalid configuration.
            details: String,
        },

        /// Some resource was exhausted, such as memory, file
        /// handles, or concurrency permits.
        ResourceExhaustion {
            // A string describing which resource was exhausted.
            resource: String,
        },

        /// An operator encountered a failure during execution.
        OperatorFailure {
            // Identifies which operator failed and possibly why.
            operator_name: String,
            // Additional details about the failure.
            reason: OperatorFailureReason,
        },
        NodeTaskPanic,
        FailedToEnqueueInitialZeroDegreeNode,
        AsyncSchedulerConfigBuilderFailure,
        ThreadPanicked,
        PoisonedLock,
        OutOfBoundsEdge {
            node_index: usize,
            node_count: usize,
        },
        TaskItemBuildFailure {
            node_index: usize,
        },
        InvalidNode {
            node_idx: usize,
        },
        OperatorFailed {
            reason: String,
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum OperatorFailureReason {
    Unknown,
}

/// A convenient result alias for network operations.
pub type NetResult<T> = Result<T, NetworkError>;
