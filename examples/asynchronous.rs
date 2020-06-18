use regex::Regex;

use memory_logger::asynchronous::MemoryLogger;


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let target = Regex::new("^mycrate::my_module")?; // optional
	let logger = MemoryLogger::setup(log::Level::Info, target)?;

	log::info!("This is a info.");
	log::warn!("This is a warning.");

	let mut reader = logger.read();

	assert!(reader.next().unwrap().contains("This is a info."));
	assert!(reader.next().unwrap().contains("This is a warning."));
	Ok(())
}
