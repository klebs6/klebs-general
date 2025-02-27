// ---------------- [ File: hydro2-async-scheduler/src/release_concurrency.rs ]
crate::ix!();

/// Releases any concurrency permit if present. This step is crucial in preventing a hang
/// if a worker forgets to release. Added explicit logs to confirm the permit is dropped.
pub fn release_concurrency<'threads, T>(
    task: &mut TaskItem<'threads, T>,
    worker_id: usize,
)
where
    T: Debug + Send + Sync + 'threads,
{
    if let Some(permit) = task.permit_mut().take() {
        eprintln!(
            "worker #{worker_id} => release_concurrency => dropping permit for node_idx={}",
            task.node_idx()
        );
        drop(permit);
    } else {
        eprintln!(
            "worker #{worker_id} => release_concurrency => no permit to drop for node_idx={}",
            task.node_idx()
        );
    }
}

#[cfg(test)]
mod release_concurrency_tests {
    use super::*;

    /// 1) Basic scenario => concurrency permit is Some => release it => now None.
    #[test]
    fn test_release_concurrency_some() {
        let mut t = mock_minimal_task_item_with_permit(0);
        assert!(
            t.permit().is_some(),
            "mock_minimal_task_item_with_permit usually gives us a real permit"
        );

        release_concurrency(&mut t, 123);
        assert!(
            t.permit().is_none(),
            "Should have dropped the permit => None now"
        );
    }

    /// 2) Already None => no crash => stays None.
    #[test]
    fn test_release_concurrency_none() {
        let mut t = mock_minimal_task_item_with_permit(0);
        // forcibly remove the permit
        *t.permit_mut() = None;
        release_concurrency(&mut t, 55);
        // still none => no crash
        assert!(t.permit().is_none());
    }

    /// 3) Real concurrency test => ensure dropping the permit frees the semaphore slot.
    ///
    /// We'll build a semaphore=1, acquire it for one task => can't acquire for second task => release => now second can acquire.
    /// That proves `release_concurrency` truly frees a slot if it was Some.
    ///
    /// This is a synchronous test, so we can't do full async or `.await`. We can do a small approach:
    ///   1) Build a semaphore=1
    ///   2) Try to `try_acquire_owned()` => success => store in `TaskItem`
    ///   3) Try to `try_acquire_owned()` again => expect None => second can't get it
    ///   4) `release_concurrency(first_task, ...) => second can now do `try_acquire_owned()` => success
    #[test]
    fn test_release_concurrency_frees_slot() {
        // Step (1) => semaphore=1
        let concurrency_limit = Arc::new(Semaphore::new(1));

        // Step (2) => first task => "acquire" a slot if possible
        let permit1: Option<OwnedSemaphorePermit> = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(permit1.is_some(), "Should succeed for the first slot");
        let mut task1 = mock_minimal_task_item_with_permit(0);
        // override the mock's default permit with ours
        *task1.permit_mut() = permit1;

        // Step (3) => second attempt => no slot left => so `try_acquire_owned()` returns None
        let permit2 = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(
            permit2.is_none(),
            "Expected None because concurrency=1 and the first slot is held by task1"
        );

        // Step (4) => release_concurrency for the first task => frees slot
        release_concurrency(&mut task1, 999);

        // Now second attempt => should succeed
        let permit2b = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(
            permit2b.is_some(),
            "Should succeed after we dropped the first permit"
        );
    }

    /// 4) concurrency=1 => if the task’s permit is None, it obviously doesn't free a slot => just no effect.
    #[test]
    fn test_release_concurrency_none_does_not_free() {
        let concurrency_limit = Arc::new(Semaphore::new(1));

        // Acquire for something else => hold it so we can't get a second
        let _someone_else = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(_someone_else.is_some());

        // Build a task with NO permit
        let mut t = mock_minimal_task_item_with_permit(123);
        *t.permit_mut() = None;
        release_concurrency(&mut t, 888);

        // concurrency still consumed by someone_else => can't reacquire
        let p2 = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(
            p2.is_none(),
            "No slot was freed => we still can't acquire"
        );
    }

    /// 5) multiple calls => if first call drops the permit => second call does nothing but also no panic
    #[test]
    fn test_release_concurrency_multiple_calls() {
        let mut t = mock_minimal_task_item_with_permit(0);

        // call #1 => frees
        release_concurrency(&mut t, 111);
        assert!(t.permit().is_none());

        // call #2 => still none => no effect
        release_concurrency(&mut t, 111);
        assert!(t.permit().is_none());
    }

    /// 6) concurrency with 2 tasks => each obtains a permit => then we drop only one => check that it frees exactly one slot
    ///    not needed if concurrency=2 => both can hold. But let's demonstrate concurrency=2 => check the final state is as expected
    #[test]
    fn test_release_concurrency_two_task_scenario() {
        let concurrency_limit = Arc::new(Semaphore::new(2));

        // Acquire for first task
        let permit_a = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(permit_a.is_some());
        let mut task_a = mock_minimal_task_item_with_permit(11);
        *task_a.permit_mut() = permit_a;

        // Acquire for second => also success => concurrency=2
        let permit_b = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(permit_b.is_some());
        let mut task_b = mock_minimal_task_item_with_permit(22);
        *task_b.permit_mut() = permit_b;

        // Now no more slot => concurrency is fully used => next try => none
        let attempt3 = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(
            attempt3.is_none(),
            "We've used 2 slots => no more left"
        );

        // drop concurrency from task_a
        release_concurrency(&mut task_a, 99);
        assert!(task_a.permit().is_none());

        // Now 1 slot is free => next attempt => success
        let attempt4 = concurrency_limit.clone().try_acquire_owned().ok();
        assert!(attempt4.is_some(), "We freed one slot => can reacquire exactly 1");

        // Meanwhile task_b is still holding its slot => no meltdown
        assert!(task_b.permit().is_some());
    }
}
