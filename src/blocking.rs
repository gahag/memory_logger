use std::{
	io::{self, Write},
	fmt::Write as _,
	ops::Deref,
	sync::{Mutex, MutexGuard},
};

use log::{Level, Log, Metadata, Record, SetLoggerError};

#[cfg(feature = "target")]
use regex::Regex;


#[derive(Debug)]
struct Logger {
	level: Level,

	#[cfg(feature = "target")]
	target: Regex,

	buffer: Mutex<String>,
}


impl Logger {
	fn lock(&self) -> MutexGuard<String> {
		self.buffer
			.lock()
			.expect("inner lock poisoned")
	}
}


impl Log for Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		#[cfg(feature = "target")]
		{
			if !self.target.is_match(metadata.target()) {
				return false;
			}
		}

		metadata.level() <= self.level
	}


	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			let target =
				if record.target().is_empty() {
					record
						.module_path()
						.unwrap_or("?")
				} else {
					record.target()
				};

			let mut buffer = self.lock();

			writeln!(
				buffer,
				"[{}] {:<5} | {}",
				target,
				record.level().to_string(),
				record.args()
			)
				.expect("std::fmt::Write should never fail for String");
		}
	}


	fn flush(&self) { }
}


/// A reference to the buffered data.
/// Note that this locks the logger, causing logging to block.
///
/// This type implements `Deref` for `str`, allowing access to the contents.
#[derive(Debug)]
pub struct BufferLockGuard<'a>(MutexGuard<'a, String>);


impl<'a> Deref for BufferLockGuard<'a> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref()
	}
}


/// A blocking memory logger. Logging and read operations may block.
///
/// You should have only a single instance of this in your program.
#[derive(Debug)]
pub struct MemoryLogger(Logger);


impl MemoryLogger {
	/// Initializes the global logger with a new MemoryLogger instance.
	/// This function should only be called once.
	///
	/// The `target` parameter is only available with the `target` feature.
	/// Only log records that match such target are enabled.
	///
	/// ```
	/// # use memory_logger::blocking::MemoryLogger;
	/// # use regex::Regex;
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let target = Regex::new("^mycrate::my_module")?; // optional
	/// let logger = MemoryLogger::setup(log::Level::Info, target)?;
	///
	/// log::info!("This is a info.");
	/// log::warn!("This is a warning.");
	/// # // Dirty hack to fix the record target defined by cargo test:
	/// # log::info!(target: "mycrate::my_module", "This is a info.");
	/// # log::warn!(target: "mycrate::my_module", "This is a warning.");
	///
	/// let mut contents = logger.read();
	///
	/// assert!(contents.contains("This is a info."));
	/// assert!(contents.contains("This is a warning."));
	/// # Ok(())
	/// # }
	/// ```
	///
	/// Returns the installed MemoryLogger instance.
	pub fn setup(
		level: Level,
		#[cfg(feature = "target")]
		target: Regex,
	) -> Result<&'static Self, SetLoggerError> {
		let logger = Box::leak(
			Box::new(
				Self(
					Logger {
						level,

						buffer: Mutex::new(String::new()),

						#[cfg(feature = "target")]
						target
					}
				)
			)
		);

		log::set_logger(&logger.0)?;

		log::set_max_level(
			level.to_level_filter()
		);

		Ok(logger)
	}


	/// Dump the contents to a writer, cleaning the buffered contents.
	pub fn dump<W>(&self, mut writer: W) -> io::Result<()>
	where
		W: Write
	{
		let buffer = &mut self.0.lock();

		writer.write_all(
			buffer.as_bytes()
		)?;

		buffer.clear();

		Ok(())
	}


	/// Gets a reference to the buffered data.
	/// Note that this locks the logger, causing logging to block.
	pub fn read(&self) -> BufferLockGuard {
		BufferLockGuard(self.0.lock())
	}


	/// Clears the log buffer.
	/// Note that this locks the logger, causing logging to block.
	pub fn clear(&self) {
		self.0.lock().clear()
	}
}
