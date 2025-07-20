#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
#![allow(clippy::uninlined_format_args)]

//! Deadpool manager for libSQL database connections.
//!
//! This crate provides a [`deadpool`] manager implementation for [`libsql`] databases,
//! supporting local, remote, and replicated database configurations.
//!
//! [`deadpool`]: https://crates.io/crates/deadpool
//! [`libsql`]: https://crates.io/crates/libsql

use std::sync::atomic::{AtomicUsize, Ordering};

use deadpool::managed::{self, RecycleError};

pub use deadpool::managed::reexports::*;
pub use libsql;

deadpool::managed_reexports!(
    "libsql",
    Manager,
    managed::Object<Manager>,
    libsql::Error,
    std::convert::Infallible
);

/// Type alias for a deadpool managed libSQL connection.
pub type Connection = managed::Object<Manager>;

/// [`Manager`] for creating and recycling libSQL [`Connection`]s.
///
/// This manager handles the lifecycle of database connections, including:
/// - Creating new connections from the underlying [`Database`]
/// - Health checking connections during recycling
/// - Ensuring connection validity before returning to the pool
///
/// [`Manager`]: managed::Manager
/// [`Connection`]: libsql::Connection
/// [`Database`]: libsql::Database
#[derive(Debug)]
pub struct Manager {
    database: libsql::Database,
    recycle_count: AtomicUsize,
}

impl Manager {
    /// Creates a new [`Manager`] using the given libSQL [`Database`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use deadpool_libsql::Manager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let database = libsql::Builder::new_local(":memory:")
    ///     .build()
    ///     .await?;
    ///
    /// let manager = Manager::from_database(database);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Database`]: libsql::Database
    #[must_use]
    pub fn from_database(database: libsql::Database) -> Self {
        Self {
            database,
            recycle_count: AtomicUsize::new(0),
        }
    }
}

impl managed::Manager for Manager {
    type Type = libsql::Connection;
    type Error = libsql::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.database.connect()
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        let recycle_count = self.recycle_count.fetch_add(1, Ordering::Relaxed) as u64;

        // Perform a simple health check to ensure the connection is still valid
        match conn.query("SELECT ?", [recycle_count]).await {
            Ok(mut rows) => match rows.next().await {
                Ok(Some(row)) => match row.get::<u64>(0) {
                    Ok(n) if n == recycle_count => Ok(()),
                    Ok(_) => Err(RecycleError::message("Recycle count mismatch")),
                    Err(e) => Err(RecycleError::message(format!(
                        "Failed to get recycle count: {}",
                        e
                    ))),
                },
                Ok(None) => Err(RecycleError::message("No rows returned from health check")),
                Err(e) => Err(RecycleError::message(format!("Failed to fetch row: {}", e))),
            },
            Err(e) => Err(RecycleError::message(format!(
                "Health check query failed: {}",
                e
            ))),
        }
    }
}
