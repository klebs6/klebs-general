// ---------------- [ File: src/reconcile_unprocessed_batch_triple.rs ]
crate::ix!();

#[async_trait]
impl<E> ReconcileUnprocessed<E> for BatchFileTriple
where BatchDownloadError: From<E>
{
    async fn reconcile_unprocessed(
        &mut self,
        client:                &dyn LanguageModelClientInterface<E>,
        expected_content_type: &ExpectedContentType,
        process_output_file_fn: &OutputFileFn,   // our new type alias
        process_error_file_fn:  &ErrorFileFn,
    ) -> Result<(), BatchReconciliationError>
    {
        info!("Attempting to reconcile unprocessed batch triple {:?}", self.index());

        let actions = BatchFileReconciliationRecommendedCourseOfAction::try_from(&*self);
        if let Err(e) = actions {
            error!("Error determining actions for batch {:?}: {:?}", self.index(), e);
            return Ok(());
        }

        let mut actions = actions.unwrap();
        info!(
            "Reconciliation actions for batch triple {:?}: {:#?}",
            self.index(),
            actions
        );

        loop {
            let steps = actions.steps();
            let mut hit_error       = false;
            let mut errors          = vec![];
            let mut updated_actions = false;

            'steps: for action in steps {
                debug!("Performing reconciliation step: {:?}", action);

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
                            debug!("Actions changed; recalculating steps");
                            break 'steps;
                        }
                    },
                    Ok(None) => {
                        trace!("No follow-up actions from step {:?}", action);
                    },
                    Err(e) => {
                        hit_error = true;
                        error!(
                            "Error applying batch action {:?} to reconcile batch {:?}: {:?}",
                            action,
                            self.index(),
                            e
                        );
                        errors.push((action.clone(), e));
                    }
                }
            }

            if updated_actions {
                // If new actions got returned, we handle them in the next iteration.
                continue;
            }

            if !hit_error {
                info!(
                    "Successfully reconciled batch triple {:?} with final actions {:#?}",
                    self.index(),
                    actions
                );
                return Ok(());
            } else {
                error!("Failed to reconcile batch triple {:?} due to errors.", self.index());
                return Err(BatchReconciliationError::ReconciliationFailed {
                    index:  self.index().clone(),
                    errors,
                });
            }
        }
    }
}
