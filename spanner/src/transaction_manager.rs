use crate::client::Error;
use crate::session::ManagedSession;
use crate::transaction::CallOptions;
use crate::transaction_rw::ReadWriteTransaction;

/// TransactionManager manages a single session for executing multiple
/// read-write transactions with session reuse. This is particularly useful
/// for manual transaction retry loops where reusing the same session helps
/// maintain lock priority across retries.
///
/// # Example
///
/// ```rust,ignore
/// use google_cloud_spanner::client::Client;
/// use google_cloud_spanner::retry::TransactionRetry;
///
/// async fn example(client: Client) -> Result<(), Status> {
///     let mut tm = client.transaction_manager().await?;
///     let retry = &mut TransactionRetry::new();
///
///     loop {
///         let tx = tm.begin_read_write_transaction().await?;
///
///         let result = do_work(tx).await;
///
///         match tx.end(result, None).await {
///             Ok((commit_result, success)) => return Ok(success),
///             Err(err) => retry.next(err).await?
///         }
///     }
/// }
/// ```
pub struct TransactionManager {
    session: Option<ManagedSession>,
    transaction: Option<ReadWriteTransaction>,
}

impl TransactionManager {
    /// Creates a new TransactionManager with the given session.
    pub(crate) fn new(session: ManagedSession) -> Self {
        Self {
            session: Some(session),
            transaction: None,
        }
    }

    /// Returns a mutable reference to the current transaction, if one has been started.
    ///
    /// Returns `None` if no transaction is currently active. Call
    /// `begin_read_write_transaction()` to start a new transaction.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut tm = client.transaction_manager().await?;
    ///
    /// // Initially returns None
    /// assert!(tm.transaction().is_none());
    ///
    /// // After begin, returns Some
    /// let tx = tm.begin_read_write_transaction().await?;
    /// assert!(tm.transaction().is_some());
    /// ```
    pub fn transaction(&mut self) -> Option<&mut ReadWriteTransaction> {
        self.transaction.as_mut()
    }

    /// Begins a new read-write transaction, reusing the session from the
    /// previous transaction if one exists. Returns a mutable reference to
    /// the transaction which can be used to execute queries and mutations.
    ///
    /// The transaction must be ended by calling `end()` on the returned
    /// reference before calling `begin_read_write_transaction()` again.
    pub async fn begin_read_write_transaction(&mut self) -> Result<&mut ReadWriteTransaction, Error> {
        self.begin_read_write_transaction_with_options(CallOptions::default(), None)
            .await
    }

    /// Begins a new read-write transaction with custom call options and transaction tag.
    /// This is similar to `begin_read_write_transaction()` but allows specifying
    /// custom options for the transaction.
    pub async fn begin_read_write_transaction_with_options(
        &mut self,
        options: CallOptions,
        transaction_tag: Option<String>,
    ) -> Result<&mut ReadWriteTransaction, Error> {
        // Extract session from previous transaction if it exists, otherwise use stored session
        let session = if let Some(ref mut tx) = self.transaction {
            tx.take_session()
                .expect("transaction should have a session")
        } else {
            self.session.take().expect("manager should have a session")
        };

        // Create new transaction with the session
        match ReadWriteTransaction::begin(session, options, transaction_tag).await {
            Ok(new_tx) => {
                // Store the transaction and return a mutable reference
                self.transaction = Some(new_tx);
                Ok(self.transaction.as_mut().unwrap())
            }
            Err(begin_error) => {
                // Store session back for next retry attempt
                self.session = Some(begin_error.session);
                Err(begin_error.status.into())
            }
        }
    }
}
