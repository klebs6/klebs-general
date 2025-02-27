// ---------------- [ File: src/reconcile_unprocessed_batch_triple.rs ]
crate::ix!();

pub async fn reconcile_unprocessed_batch_triple<OutputF,ErrorF,OFut,EFut>(
    triple:                 &mut BatchFileTriple,
    client:                 &OpenAIClientHandle,
    expected_content_type:  &ExpectedContentType,
    process_output_file_fn: &OutputF,
    process_error_file_fn:  &ErrorF,

) -> Result<(), BatchReconciliationError> 
where
    OutputF: Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:  Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:    Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:    Future<Output = Result<(), BatchErrorProcessingError>> + Send,
{

    let actions = BatchFileReconciliationRecommendedCourseOfAction::try_from(&*triple);

    if let Err(e) = actions {
        error!("Error determining actions for batch {:?}: {:?}", triple.index(), e);
        // We just want to log and return
        return Ok(());
    }

    let mut actions = actions.unwrap();

    info!("reconciling unprocessed batch triple {:#?} with actions {:#?}", triple, actions);

    loop {
        let steps = actions.steps();

        let mut hit_error       = false;
        let mut errors          = vec![];
        let mut updated_actions = false;

        'steps: for action in steps {

            match execute_reconciliation_operation(
                triple,
                client,
                action,
                expected_content_type,
                process_output_file_fn,
                process_error_file_fn
            ).await {
                Ok(Some(new_actions)) => {
                    if actions != new_actions {
                        actions = new_actions;
                        updated_actions = true;
                        break 'steps;
                    }
                },
                Ok(None) => { /* No action needed */ },
                Err(e) => {
                    hit_error = true;
                    error!(
                        "Error applying batch action {:?} to reconcile the batch {:?}: {:?}",
                        action, 
                        triple.index(), 
                        e
                    );
                    errors.push((action.clone(),e));
                }
            }
        }

        if updated_actions {
            continue;
        }

        if !hit_error {
            info!(
                "Reconciled batch with actions. batch={:?}: actions={:?}",
                triple.index(),
                actions
            );
            return Ok(());
        } else {
            // Exit the loop if we can't make progress due to errors
            error!(
                "Failed to reconcile batch {:?} due to errors.",
                triple.index()
            );
            return Err(BatchReconciliationError::ReconciliationFailed {
                index: triple.index().clone(),
                errors
            });
        }
    }
}
