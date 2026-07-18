use crate::{fans, pci, process, storage, temps, usb};

pub fn collect() -> Result<(), String> {
    storage::collect()?;
    temps::collect()?;
    fans::collect()?;
    usb::collect()?;
    pci::collect()?;
    process::collect()?;
    Ok(())
}
