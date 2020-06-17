use std::io::{self, Write};

use log::{Level, Log, Metadata, Record, SetLoggerError};

use flume::{Sender, Receiver};

#[cfg(feature = "target")]
use regex::Regex;


struct Logger {
	level: Level,

	#[cfg(feature = "target")]
	target: Regex,

	tx: Sender<Box<str>>,
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

			self.tx.send(
				format!(
					"[{}] {:<5} | {}",
					target,
					record.level().to_string(),
					record.args()
				)
					.into_boxed_str()
			)
				.expect("channel should not be closed");
		}
	}


	fn flush(&self) { }
}


impl std::fmt::Debug for Logger {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut debug_struct = f.debug_struct("Logger");

		debug_struct.field("level", &self.level);

		#[cfg(feature = "target")]
		{
			debug_struct.field("target", &self.target);
		}

		debug_struct.finish()
	}
}


/// An asynchronous memory logger. Logging and read operations never block.
///
/// You should have only a single instance of this in your program.
pub struct MemoryLogger {
	logger: Logger,
	// Receiver is not (Sync + Send), which is required by the Log trait.
	// Therefore, we implement Log just for the Logger struct.
	rx: Receiver<Box<str>>,
}


impl MemoryLogger {
	/// Initializes the global logger with a new MemoryLogger instance.
	/// This function should only be called once.
	///
	/// The `target` parameter is only available with the `target` feature.
	/// Only log records that match such target are enabled.
	///
	/// ```
	/// # use memory_logger::asynchronous::MemoryLogger;
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
	/// let mut reader = logger.read();
	///
	/// assert!(reader.next().unwrap().contains("This is a info."));
	/// assert!(reader.next().unwrap().contains("This is a warning."));
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
		let (tx, rx) = flume::unbounded();

		let logger = Box::leak(
			Box::new(
				Self {
					logger: Logger {
						level,

						#[cfg(feature = "target")]
						target,

						tx,
					},

					rx,
				}
			)
		);

		log::set_logger(&logger.logger)?;

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
		for record in self.rx.try_iter() {
			writer.write_all(
				record.as_bytes()
			)?;

			writeln!(writer)?;
		}

		Ok(())
	}


	/// Gets an iterator to the buffered entries.
	///
	/// This iterator will consume the entries. If you wish to iterate twice, you must
	/// collect.
	pub fn read<'a>(&'a self) -> impl Iterator<Item = Box<str>> + 'a {
		self.rx.try_iter()
	}
}


impl std::fmt::Debug for MemoryLogger {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("MemoryLogger")
			.field("logger", &self.logger)
			.finish()
	}
}
