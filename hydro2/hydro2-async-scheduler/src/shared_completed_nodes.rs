// ---------------- [ File: src/shared_completed_nodes.rs ]
crate::ix!();

//========================================
// InsertOutcome
//========================================

/// Outcome when inserting a node index into `SharedCompletedNodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    /// This index was not previously present; now inserted.
    Inserted,
    /// This index was already in the set; no change.
    Duplicate,
}

//========================================
// SharedCompletedNodes Implementation
//========================================

/// A concurrency‐safe set of completed node indices, using `tokio::sync::AsyncMutex<HashSet<usize>>`.
#[derive(Clone)]
pub struct SharedCompletedNodes {
    inner: Arc<AsyncMutex<HashSet<usize>>>,
}

impl Debug for SharedCompletedNodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // We don't lock and dump all indices here to avoid potential deadlocks.
        write!(f, "SharedCompletedNodes {{ ... }}")
    }
}

impl SharedCompletedNodes {
    /// Constructs an empty `SharedCompletedNodes`.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(HashSet::new())),
        }
    }

    /// Constructs a `SharedCompletedNodes` by inserting all elements from the given slice.
    /// Duplicates in the slice are ignored (only one copy of each index is stored).
    pub fn from_slice(nodes: &[usize]) -> Self {
        let mut set = HashSet::with_capacity(nodes.len());
        for &n in nodes {
            set.insert(n);
        }
        SharedCompletedNodes {
            inner: Arc::new(AsyncMutex::new(set)),
        }
    }

    /// Inserts `node_idx` into the set. Returns `InsertOutcome` or a potential `NetworkError`.
    pub async fn insert(&self, node_idx: usize) -> Result<InsertOutcome, NetworkError> {
        // For example, you could enforce a maximum or valid range:
        // if node_idx > 100_000 { return Err(NetworkError::InvalidNode { node_idx }); }

        let mut guard = self.inner.lock().await;
        if guard.contains(&node_idx) {
            Ok(InsertOutcome::Duplicate)
        } else {
            guard.insert(node_idx);
            Ok(InsertOutcome::Inserted)
        }
    }

    /// Returns the current number of unique completed nodes.
    pub async fn len(&self) -> usize {
        let guard = self.inner.lock().await;
        guard.len()
    }

    /// Checks if `node_idx` is in the set.
    pub async fn contains(&self, node_idx: usize) -> bool {
        let guard = self.inner.lock().await;
        guard.contains(&node_idx)
    }

    /// Clears all entries. Returns the number of entries that were cleared.
    pub async fn clear(&self) -> usize {
        let mut guard = self.inner.lock().await;
        let old_len = guard.len();
        guard.clear();
        old_len
    }

    /// Returns `true` if we have exactly `total_count` unique nodes in the set.
    pub async fn is_all_done(&self, total_count: usize) -> bool {
        self.len().await == total_count
    }

    /// Returns a sorted snapshot (as a `Vec<usize>`) of the current contents.
    pub async fn as_slice(&self) -> Vec<usize> {
        let guard = self.inner.lock().await;
        let mut v: Vec<usize> = guard.iter().copied().collect();
        v.sort_unstable();
        v
    }

    /// Marks `node_idx` as completed, logs the outcome, and optionally calls a checkpoint callback.
    ///
    /// # Arguments
    /// * `node_idx`: The index to record.
    /// * `worker_id`: For logging; e.g. which worker thread is marking completion.
    /// * `checkpoint_cb`: Optionally, a callback to be invoked with a sorted snapshot of the set.
    pub async fn mark_node_completed(
        &self,
        node_idx: usize,
        worker_id: usize,
        checkpoint_cb: Option<Arc<dyn CheckpointCallback>>,
    ) {
        eprintln!(
            "worker #{worker_id} => mark_node_completed => about to insert node_idx={}",
            node_idx
        );

        let outcome = match self.insert(node_idx).await {
            Ok(o) => o,
            Err(e) => {
                eprintln!(
                    "worker #{worker_id} => ERROR inserting node_idx={} => {:?}",
                    node_idx, e
                );
                return;
            }
        };

        match outcome {
            InsertOutcome::Inserted => {
                eprintln!(
                    "worker #{worker_id} => mark_node_completed => node_idx={} => newly inserted",
                    node_idx
                );
            }
            InsertOutcome::Duplicate => {
                eprintln!(
                    "worker #{worker_id} => mark_node_completed => node_idx={} => DUPLICATE! Already present",
                    node_idx
                );
            }
        }

        // Snapshot for logging + checkpoint
        let snapshot = self.as_slice().await;
        eprintln!(
            "worker #{worker_id} => mark_node_completed => now completed_nodes={:?}",
            snapshot
        );

        // If we have a checkpoint callback => invoke it
        if let Some(cb) = checkpoint_cb {
            eprintln!(
                "worker #{worker_id} => invoking checkpoint callback => node_idx={}",
                node_idx
            );
            if let Err(err) = cb.checkpoint(&snapshot).await {
                eprintln!(
                    "worker #{worker_id} => checkpoint callback error => {:?} => ignoring",
                    err
                );
            } else {
                eprintln!(
                    "worker #{worker_id} => checkpoint callback returned => node_idx={}",
                    node_idx
                );
            }
        }
    }
}

/// Convenience `From<&[usize]>` so you can do:
/// `SharedCompletedNodes::from(&[0,1,2])` directly.
impl From<&[usize]> for SharedCompletedNodes {
    fn from(slice: &[usize]) -> Self {
        Self::from_slice(slice)
    }
}

impl<const N: usize> From<&[usize; N]> for SharedCompletedNodes {
    fn from(array: &[usize; N]) -> Self {
        Self::from_slice(array)
    }
}

//============================================================
// Test Suite
//============================================================

#[cfg(test)]
mod shared_completed_nodes_tests {
    use super::*;

    // A basic test verifying that from_slice works, duplicates are collapsed, etc.
    #[traced_test]
    async fn test_from_slice() {
        let scn = SharedCompletedNodes::from_slice(&[0, 1, 2, 2, 1]);
        assert_eq!(scn.len().await, 3);

        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![0, 1, 2]);
    }

    // Using `From<&[usize]>`
    #[traced_test]
    async fn test_from_trait_impl() {
        // This slice has duplicates => unique final set is {5,7,8,9}
        let scn = SharedCompletedNodes::from(&[5, 5, 7, 8, 7, 9][..]);
        assert_eq!(scn.len().await, 4);

        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![5, 7, 8, 9]);
    }

    // Basic single-threaded usage of insert + is_all_done
    #[traced_test]
    async fn test_basic_insert_and_is_all_done() {
        let scn = SharedCompletedNodes::new();
        assert_eq!(scn.len().await, 0);
        assert!(!scn.is_all_done(3).await);

        // Insert node=10
        let out1 = scn.insert(10).await.unwrap();
        assert_eq!(out1, InsertOutcome::Inserted);
        assert!(scn.contains(10).await);
        assert_eq!(scn.len().await, 1);

        // Insert same node=10 again => Duplicate
        let out2 = scn.insert(10).await.unwrap();
        assert_eq!(out2, InsertOutcome::Duplicate);
        assert_eq!(scn.len().await, 1);

        // Insert node=11
        scn.insert(11).await.unwrap();
        assert_eq!(scn.len().await, 2);

        // is_all_done(3) => false
        assert!(!scn.is_all_done(3).await);
        // Insert node=12 => now we have 3
        scn.insert(12).await.unwrap();
        assert!(scn.is_all_done(3).await);
    }

    // Tests concurrency inserting the *same* node => exactly 1 Inserted, rest Duplicate
    #[traced_test]
    async fn test_concurrent_duplicates() {
        let scn = Arc::new(SharedCompletedNodes::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let scn_clone = scn.clone();
            handles.push(tokio::task::spawn(async move {
                scn_clone.insert(7).await
            }));
        }

        // Wait for all tasks
        let results = futures::future::join_all(handles).await;

        // We expect exactly 1 InsertOutcome::Inserted, 9 InsertOutcome::Duplicate
        let mut inserted_count = 0;
        let mut duplicate_count = 0;
        for res in results {
            match res {
                Ok(Ok(InsertOutcome::Inserted)) => inserted_count += 1,
                Ok(Ok(InsertOutcome::Duplicate)) => duplicate_count += 1,
                Ok(Err(e)) => panic!("Insertion returned error: {:?}", e),
                Err(join_err) => panic!("Task join error: {:?}", join_err),
            }
        }
        assert_eq!(inserted_count, 1);
        assert_eq!(duplicate_count, 9);

        // Should contain exactly one unique node
        assert_eq!(scn.len().await, 1);
        assert!(scn.contains(7).await);
    }

    // Tests concurrency with distinct node values => all Inserted, no duplicates
    #[traced_test]
    async fn test_concurrent_distinct_inserts() {
        let scn = Arc::new(SharedCompletedNodes::new());

        let mut handles = vec![];
        for i in 1..=10 {
            let scn_clone = scn.clone();
            handles.push(tokio::task::spawn(async move {
                scn_clone.insert(i).await
            }));
        }
        let results = futures::future::join_all(handles).await;

        let mut inserted_count = 0;
        for res in results {
            match res {
                Ok(Ok(InsertOutcome::Inserted)) => inserted_count += 1,
                Ok(Ok(InsertOutcome::Duplicate)) => panic!("Unexpected duplicate in distinct inserts"),
                Ok(Err(e)) => panic!("Insertion error: {:?}", e),
                Err(join_err) => panic!("Task join error: {:?}", join_err),
            }
        }
        // All 10 => Inserted
        assert_eq!(inserted_count, 10);

        let final_len = scn.len().await;
        assert_eq!(final_len, 10);
    }

    // Testing `clear()`
    #[traced_test]
    async fn test_clear() {
        let scn = SharedCompletedNodes::from_slice(&[1, 2, 3]);
        assert_eq!(scn.len().await, 3);

        let old_len = scn.clear().await;
        assert_eq!(old_len, 3);
        assert_eq!(scn.len().await, 0);
    }

    // Optional out-of-range node test. If you want to enforce range checks, uncomment in `insert()`.
    #[traced_test]
    async fn test_out_of_range_node() {
        let scn = SharedCompletedNodes::new();
        // If the code in `insert(...)` is uncommented:
        // let res = scn.insert(999999).await;
        // match res {
        //     Err(NetworkError::InvalidNode { node_idx }) => assert_eq!(node_idx, 999999),
        //     _ => panic!("Expected an out-of-range error"),
        // }

        // By default, we do no checks => it simply inserts
        let res = scn.insert(999999).await.unwrap();
        assert_eq!(res, InsertOutcome::Inserted);
        assert!(scn.contains(999999).await);
    }

    // A random concurrency stress test => many inserts from 0..=10, repeated 100 times.
    #[traced_test]
    async fn test_stress_concurrent_mixture() {
        let scn = Arc::new(SharedCompletedNodes::new());
        let mut handles = vec![];

        use rand::Rng;
        for _ in 0..100 {
            let scn_clone = scn.clone();
            handles.push(tokio::task::spawn(async move {
                let random_idx = rand::thread_rng().gen_range(0..=10);
                scn_clone.insert(random_idx).await
            }));
        }

        let results = futures::future::join_all(handles).await;
        for r in results {
            let _ = r.expect("Join error").expect("Insertion error");
        }
        // final len should be <= 11
        let final_len = scn.len().await;
        assert!(final_len <= 11);
    }

    //====================================================
    // Tests for `mark_node_completed` with callbacks
    //====================================================

    /// A checkpoint callback that just records whether it was invoked
    /// and the last snapshot of completed nodes it saw.
    #[derive(Debug)]
    struct MockCheckpoint {
        invoked: Arc<AsyncMutex<bool>>,
        last_completed: Arc<AsyncMutex<Option<Vec<usize>>>>,
    }

    #[async_trait]
    impl CheckpointCallback for MockCheckpoint {
        async fn checkpoint(&self, completed_nodes: &[usize]) -> Result<(), NetworkError> {
            let mut ivk = self.invoked.lock().await;
            *ivk = true;
            let mut snap = self.last_completed.lock().await;
            *snap = Some(completed_nodes.to_vec());
            Ok(())
        }
    }

    impl MockCheckpoint {
        fn new() -> Self {
            Self {
                invoked: Arc::new(AsyncMutex::new(false)),
                last_completed: Arc::new(AsyncMutex::new(None)),
            }
        }

        async fn was_invoked(&self) -> bool {
            *self.invoked.lock().await
        }

        async fn last_snapshot(&self) -> Option<Vec<usize>> {
            self.last_completed.lock().await.clone()
        }
    }

    #[traced_test]
    async fn test_mark_node_completed_no_callback() {
        let scn = SharedCompletedNodes::new();
        scn.mark_node_completed(7, 123, None).await;

        // final => node=7
        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![7]);
    }

    #[traced_test]
    async fn test_mark_node_completed_with_callback() {
        let scn = SharedCompletedNodes::new();
        let cp = Arc::new(MockCheckpoint::new());

        scn.mark_node_completed(42, 999, Some(cp.clone())).await;
        // Check we have {42}
        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![42]);

        // Callback was invoked => last_snapshot => [42]
        assert!(cp.was_invoked().await);
        assert_eq!(cp.last_snapshot().await, Some(vec![42]));
    }

    #[traced_test]
    async fn test_mark_node_completed_twice_same_node() {
        let scn = SharedCompletedNodes::new();
        scn.mark_node_completed(0, 1, None).await;
        // mark again => duplicate
        scn.mark_node_completed(0, 1, None).await;

        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![0]);
    }

    #[traced_test]
    async fn test_mark_node_completed_many_distinct_nodes() {
        let scn = SharedCompletedNodes::new();
        for i in 0..5 {
            scn.mark_node_completed(i, 77, None).await;
        }
        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![0,1,2,3,4]);
    }

    /// Test concurrency => multiple tasks calling mark_node_completed on the same node
    #[traced_test]
    async fn test_mark_node_completed_concurrency_duplicate() {
        let scn = Arc::new(SharedCompletedNodes::new());
        let mut handles = vec![];
        for _ in 0..10 {
            let scn_clone = scn.clone();
            handles.push(tokio::spawn(async move {
                // same node=99
                scn_clone.mark_node_completed(99, 666, None).await;
            }));
        }

        futures::future::join_all(handles).await;

        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![99]);
    }

    #[derive(Debug)]
    struct FailingCheckpoint;

    #[async_trait]
    impl CheckpointCallback for FailingCheckpoint {
        async fn checkpoint(&self, _completed_nodes: &[usize]) -> Result<(), NetworkError> {
            Err(NetworkError::InvalidNode { node_idx: 999 })
        }
    }

    #[traced_test]
    async fn test_mark_node_completed_callback_error_ignored() {
        let scn = SharedCompletedNodes::new();
        let cb = Arc::new(FailingCheckpoint);

        scn.mark_node_completed(77, 123, Some(cb)).await;
        // Despite error, we still have 77 in the set
        let snap = scn.as_slice().await;
        assert_eq!(snap, vec![77]);
    }

    /// Mix scenario => some nodes with callbacks, some without
    #[traced_test]
    async fn test_mark_node_completed_mixed_scenario() {
        let scn = Arc::new(SharedCompletedNodes::new());
        let cp = Arc::new(MockCheckpoint::new());

        let mut futs = Vec::new();
        for i in 0..5 {
            let scn_clone = scn.clone();
            let maybe_cb = if i % 2 == 0 {
                Some(cp.clone() as Arc<dyn CheckpointCallback>)
            } else {
                None
            };
            futs.push(tokio::spawn(async move {
                scn_clone.mark_node_completed(i, 888, maybe_cb).await;
            }));
        }

        futures::future::join_all(futs).await;

        let snap = scn.as_slice().await;
        // {0,1,2,3,4}
        assert_eq!(snap, vec![0,1,2,3,4]);
        // Callback was used for nodes 0,2,4 => it should have been invoked at least once
        assert!(cp.was_invoked().await);
        // The last snapshot likely is [0,1,2,3,4]
        let final_snap = cp.last_snapshot().await.unwrap();
        assert_eq!(final_snap, vec![0,1,2,3,4]);
    }
}
