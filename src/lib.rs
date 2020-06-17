/*!
A logger that stores log entries in memory, allowing late consumption.

There are two mutually exclusive flavors, which can be selected through features:
- blocking: A single buffer is shared through a mutex, therefore operations may block.
- asynchronous: Entries are managed through a channel, therefore operations may never block.

One should **not** attempt to use both flavors simultaneously.

# Features
- `blocking`: enables the `blocking` module. Should not be used with `asynchronous`.
- `asynchronous`: enables the `asynchronous` module. Should not be used with `blocking`.
- `target`: enables the `target` regex parameter for both flavors, allowing filtering
  logs by target (module name).
*/

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "asynchronous")]
pub mod asynchronous;
