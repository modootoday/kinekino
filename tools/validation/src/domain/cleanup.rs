pub const TARGET: &str = "runtime/validation-vm";

pub fn owned_entry(relative: &str, directory: bool) -> bool {
    if relative
        .split('/')
        .any(|part| part == ".." || part.is_empty())
    {
        return false;
    }
    if directory {
        return matches!(
            relative,
            "Source-DPI-Lab"
                | "Source-DPI-Lab/Virtual Machines"
                | "media"
                | "media/vdd-control-25.7.23"
        ) || relative.starts_with("media/vdd-control-25.7.23/");
    }
    matches!(
        relative,
        "Source-DPI-Lab/os.vhdx"
            | "media/noprompt-provenance.json"
            | "media/vdd-control-25.7.23.zip"
            | "media/windows11-enterprise-eval-ko-kr.iso"
            | "media/windows11-enterprise-eval-ko-kr-noprompt.iso"
    ) || relative.starts_with("media/vdd-control-25.7.23/")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_rejects_unknown_assets_and_parent_traversal() {
        assert!(owned_entry("Source-DPI-Lab/os.vhdx", false));
        assert!(owned_entry(
            "media/vdd-control-25.7.23/Dependencies/devcon.exe",
            false
        ));
        assert!(!owned_entry("media/private-backup.iso", false));
        assert!(!owned_entry("Source-DPI-Lab/photos", true));
        assert!(!owned_entry("media/vdd-control-25.7.23/../backup", false));
    }
}
