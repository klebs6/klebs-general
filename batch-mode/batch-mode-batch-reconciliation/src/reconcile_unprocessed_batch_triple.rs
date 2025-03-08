// ---------------- [ File: src/reconcile_unprocessed_batch_triple.rs ]
crate::ix!();

#[async_trait]
impl<OutputF,ErrorF,OFut,EFut> ReconcileUnprocessed<OutputF,ErrorF,OFut,EFut> 
for BatchFileTriple 
where
    OutputF: Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:  Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:    Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:    Future<Output = Result<(), BatchErrorProcessingError>> + Send,
{
    async fn reconcile_unprocessed(
        &mut self,
        client:                 &OpenAIClientHandle,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputF,
        process_error_file_fn:  &ErrorF,

    ) -> Result<(), BatchReconciliationError> 
    {
        let actions = BatchFileReconciliationRecommendedCourseOfAction::try_from(&*self);

        if let Err(e) = actions {
            error!("Error determining actions for batch {:?}: {:?}", self.index(), e);
            // We just want to log and return
            return Ok(());
        }

        let mut actions = actions.unwrap();

        info!("reconciling unprocessed batch triple {:#?} with actions {:#?}", self, actions);

        loop {
            let steps = actions.steps();

            let mut hit_error       = false;
            let mut errors          = vec![];
            let mut updated_actions = false;

            'steps: for action in steps {

                match self.execute_reconciliation_operation(
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
                            self.index(), 
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
                    self.index(),
                    actions
                );
                return Ok(());
            } else {
                // Exit the loop if we can't make progress due to errors
                error!(
                    "Failed to reconcile batch {:?} due to errors.",
                    self.index()
                );
                return Err(BatchReconciliationError::ReconciliationFailed {
                    index: self.index().clone(),
                    errors
                });
            }
        }
    }
}
