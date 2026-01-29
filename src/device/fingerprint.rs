use crate::error::{AeroBaseError, Result};
use serde_json::json;
use sha2::{Digest, Sha256};
use sysinfo::System;

pub fn generate_fingerprint() -> Result<String> {
    let mut hasher = Sha256::new();
    if let Ok(machine_id) = machine_uid::get() {
        hasher.update(machine_id.as_bytes());
    } else {
        let sys = System::new_all();
        if let Some(name) = System::name() {
            hasher.update(name.as_bytes());
        }
        if let Some(os_version) = System::os_version() {
            hasher.update(os_version.as_bytes());
        }
        if let Some(kernel) = System::kernel_version() {
            hasher.update(kernel.as_bytes());
        }
        if let Some(hostname) = System::host_name() {
            hasher.update(hostname.as_bytes());
        }
        hasher.update(sys.cpus().len().to_string().as_bytes());
        if let Some(cpu) = sys.cpus().first() {
            hasher.update(cpu.brand().as_bytes());
        }
        hasher.update(sys.total_memory().to_string().as_bytes());
    }
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn get_hardware_info() -> Result<String> {
    let sys = System::new_all();
    
    let info = json!({
        "system_name": System::name(),
        "os_version": System::os_version(),
        "kernel_version": System::kernel_version(),
        "host_name": System::host_name(),
        "cpu_count": sys.cpus().len(),
        "cpu_brand": sys.cpus().first().map(|c| c.brand()),
        "total_memory_mb": sys.total_memory() / 1024 / 1024,
        "architecture": std::env::consts::ARCH,
        "os_family": std::env::consts::FAMILY,
    });
    
    serde_json::to_string(&info).map_err(|e| AeroBaseError::Serialization(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_fingerprint() {
        let fp1 = generate_fingerprint().unwrap();
        let fp2 = generate_fingerprint().unwrap();
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64);
    }

    #[test]
    fn test_hardware_info() {
        let info = get_hardware_info().unwrap();
        assert!(!info.is_empty());
        let parsed: serde_json::Value = serde_json::from_str(&info).unwrap();
        assert!(parsed.is_object());
    }
}
