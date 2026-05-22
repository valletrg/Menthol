//! IO tasks for file transfers.
//!
//! Each active transfer uses a pair of tasks:
//! - Download: SocketReader + DiskWriter (mpsc channel between)
//! - Upload: DiskReader + SocketWriter (mpsc channel between)
//!
//! Backpressure is built into the bounded channels: if the consumer is slow,
//! the producer pauses automatically.

pub mod bandwidth;
pub mod disk_reader;
pub mod disk_writer;
pub mod reader;
pub mod writer;

pub use bandwidth::TokenBucket;
pub use disk_reader::disk_reader_task;
pub use disk_writer::disk_writer_task;
pub use reader::socket_reader_task;
pub use writer::socket_writer_task;
