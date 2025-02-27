// ---------------- [ File: hydro2-async-scheduler/src/create_worker_channels.rs ]
crate::ix!();

pub fn create_worker_channels<'threads, T>(num_workers: usize, buffer_size: usize) 
-> (Vec<mpsc::Sender<TaskItem<'threads, T>>>, Vec<mpsc::Receiver<TaskItem<'threads, T>>>) 
where
    T: Debug + Send + Sync + 'threads,
{
    let mut worker_senders   = Vec::with_capacity(num_workers);
    let mut worker_receivers = Vec::with_capacity(num_workers);

    for w in 0..num_workers {
        let (tx, rx) = mpsc::channel::<TaskItem<'threads, T>>(buffer_size);
        worker_senders.push(tx);
        worker_receivers.push(rx);
        eprintln!("created worker channel for worker #{}", w);
    }

    (worker_senders, worker_receivers)
}

#[cfg(test)]
mod create_worker_channels_tests {
    use super::*;
    use crate::mpsc::error::TryRecvError;

    #[traced_test]
    async fn test_create_worker_channels_initialization() {
        let num_workers = 4;
        let buffer_size = 10;

        let (worker_senders, worker_receivers) = create_worker_channels::<()>(num_workers, buffer_size);

        assert_eq!(worker_senders.len(), num_workers);
        assert_eq!(worker_receivers.len(), num_workers);
    }

    #[traced_test]
    async fn test_worker_channels_can_send_and_receive() {
        let num_workers = 2;
        let buffer_size = 10;
        let (worker_senders, mut worker_receivers) = create_worker_channels::<()>(num_workers, buffer_size);

        let network             = Arc::new(AsyncMutex::new(Network::default()));
        let shared_in_degs      = Arc::new(AsyncMutex::new(vec![]));
        let completed_nodes     = SharedCompletedNodes::new();
        let (child_nodes_tx, _) = mpsc::channel(10);
        let (ready_nodes_tx, _) = mpsc::channel(10);

        let task = task_item!(
            node_idx:        42_usize,
            permit:          None,
            network:         network,
            shared_in_degs:  shared_in_degs,
            output_tx:       None,
            checkpoint_cb:   None,
            child_nodes_tx:  child_nodes_tx,
            ready_nodes_tx:  ready_nodes_tx,
            completed_nodes: completed_nodes
        );

        worker_senders[0].send(task).await.expect("Failed to send task");

        let received_task = worker_receivers[0].recv().await.expect("Failed to receive task");
        assert_eq!(*received_task.node_idx(), 42);
    }

    #[traced_test]
    async fn test_no_message_in_empty_channel() {
        let num_workers = 2;
        let buffer_size = 10;

        // Ensure the senders are kept in scope until the end of the test
        let (worker_senders, mut worker_receivers) = create_worker_channels::<()>(num_workers, buffer_size);

        // Validate the channel is empty but not disconnected
        match worker_receivers[0].try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => panic!("The sender was disconnected unexpectedly."),
            _ => panic!("Channel should be empty but was not"),
        }

        // Explicitly drop senders after the validation to ensure the channel isn't closed prematurely
        drop(worker_senders);
    }
}
