use regex::Regex;

use memory_logger::blocking::MemoryLogger;


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let target = Regex::new("^mycrate::my_module")?; // optional
	let logger = MemoryLogger::setup(log::Level::Info, target)?;

	log::info!("This is a info.");
	log::warn!("This is a warning.");

	let contents = logger.read();

	assert!(contents.contains("This is a info."));
	assert!(contents.contains("This is a warning."));
	Ok(())
}
