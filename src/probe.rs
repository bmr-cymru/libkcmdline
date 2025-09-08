use std::collections::HashSet;
use std::fs;

pub struct SystemProbe {
    pub hardware_tags: HashSet<String>,
    pub software_tags: HashSet<String>,
    pub distribution: DistributionInfo,
}

pub struct HardwareProbe {
    tags: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct DistributionInfo {
    pub id: String,
    pub version_id: String,
    pub variant_id: Option<String>,
    pub name: String,
}

impl SystemProbe {
    pub fn new() -> Result<Self, crate::error::ProbeError> {
        let mut hardware_tags = HashSet::new();
        let mut software_tags = HashSet::new();

        // Architecture
        hardware_tags.insert(format!("arch:{}", std::env::consts::ARCH));

        // Hardware probing
        hardware_tags.extend(Self::probe_cpu_features()?);
        hardware_tags.extend(Self::probe_pci_devices()?);
        hardware_tags.extend(Self::probe_dmi()?);

        // Software probing
        software_tags.extend(Self::probe_init_system()?);
        software_tags.extend(Self::probe_initramfs_generator()?);

        let distribution = Self::probe_distribution()?;

        Ok(Self {
            hardware_tags,
            software_tags,
            distribution,
        })
    }

    fn probe_cpu_features() -> Result<Vec<String>, crate::error::ProbeError> {
        // Read /proc/cpuinfo for features
        todo!()
    }

    fn probe_pci_devices() -> Result<Vec<String>, crate::error::ProbeError> {
        // Read /sys/bus/pci/devices/*/vendor and device
        todo!()
    }

    fn probe_dmi() -> Result<Vec<String>, crate::error::ProbeError> {
        // Read /sys/class/dmi/id/*
        todo!()
    }

    fn probe_init_system() -> Result<Vec<String>, crate::error::ProbeError> {
        // Check /proc/1/comm, systemctl --version, etc.
        todo!()
    }

    fn probe_initramfs_generator() -> Result<Vec<String>, crate::error::ProbeError> {
        // Check for dracut, initramfs-tools, mkinitcpio
        todo!()
    }

    fn probe_distribution() -> Result<DistributionInfo, crate::error::ProbeError> {
        // Parse /etc/os-release
        let _os_release = fs::read_to_string("/etc/os-release")?;
        // Parse and extract distribution info
        todo!()
    }
}
