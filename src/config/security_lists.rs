//! Security lists data structures
//!
//! Defines the structure for parsing security-lists.toml

#![allow(dead_code)]

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Root structure for security-lists.toml
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityLists {
    pub meta: SecurityListsMeta,
    pub sensitive_paths: SensitivePaths,
    pub dangerous_commands: DangerousCommands,
    pub system_read_paths: SystemReadPaths,
}

/// Metadata for security lists (version tracking, downgrade protection)
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityListsMeta {
    /// Monotonic version number (must always increase)
    pub version: u64,
    /// Schema version for format compatibility
    pub schema_version: String,
}

/// Sensitive paths organized by category
#[derive(Debug, Clone, Deserialize)]
pub struct SensitivePaths {
    #[serde(default)]
    pub ssh: Vec<String>,
    #[serde(default)]
    pub aws: Vec<String>,
    #[serde(default)]
    pub gcp: Vec<String>,
    #[serde(default)]
    pub azure: Vec<String>,
    #[serde(default)]
    pub kubernetes: Vec<String>,
    #[serde(default)]
    pub docker: Vec<String>,
    #[serde(default)]
    pub gnupg: Vec<String>,
    #[serde(default)]
    pub keychain: Vec<String>,
    #[serde(default)]
    pub password_store: Vec<String>,
    #[serde(default)]
    pub onepassword: Vec<String>,
    #[serde(default)]
    pub macos_private: Vec<String>,
    #[serde(default)]
    pub browser_data: Vec<String>,
    #[serde(default)]
    pub credential_files: Vec<String>,
    #[serde(default)]
    pub secrets_dirs: Vec<String>,
    #[serde(default)]
    pub shell_configs: Vec<String>,
    #[serde(default)]
    pub history_files: Vec<String>,
}

/// Dangerous commands organized by category
#[derive(Debug, Clone, Deserialize)]
pub struct DangerousCommands {
    #[serde(default)]
    pub file_destruction: Vec<String>,
    #[serde(default)]
    pub disk_destruction: Vec<String>,
    #[serde(default)]
    pub permission_chaos: Vec<String>,
    #[serde(default)]
    pub system_modification: Vec<String>,
    #[serde(default)]
    pub package_managers: Vec<String>,
    #[serde(default)]
    pub dangerous_file_ops: Vec<String>,
    #[serde(default)]
    pub network_exfiltration: Vec<String>,
    #[serde(default)]
    pub arbitrary_execution: Vec<String>,
    #[serde(default)]
    pub privilege_escalation: Vec<String>,
}

/// System read paths needed for executables
#[derive(Debug, Clone, Deserialize)]
pub struct SystemReadPaths {
    #[serde(default)]
    pub common: Vec<String>,
    #[serde(default)]
    pub linux: LinuxSystemPaths,
    #[serde(default)]
    pub macos: MacosSystemPaths,
}

/// Linux-specific system paths
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LinuxSystemPaths {
    #[serde(default)]
    pub libraries: Vec<String>,
    #[serde(default)]
    pub linker: Vec<String>,
    #[serde(default)]
    pub config: Vec<String>,
    #[serde(default)]
    pub locale: Vec<String>,
    #[serde(default)]
    pub ssl: Vec<String>,
    #[serde(default)]
    pub terminfo: Vec<String>,
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default)]
    pub proc: Vec<String>,
    #[serde(default)]
    pub nix: Vec<String>,
    #[serde(default)]
    pub tmp: Vec<String>,
}

/// macOS-specific system paths
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MacosSystemPaths {
    #[serde(default)]
    pub executables: Vec<String>,
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub dyld: Vec<String>,
    #[serde(default)]
    pub ssl: Vec<String>,
    #[serde(default)]
    pub locale: Vec<String>,
    #[serde(default)]
    pub terminfo: Vec<String>,
    #[serde(default)]
    pub system: Vec<String>,
    #[serde(default)]
    pub user_library: Vec<String>,
    #[serde(default)]
    pub user_local: Vec<String>,
    #[serde(default)]
    pub writable: Vec<String>,
}

impl SecurityLists {
    /// Get all sensitive paths as a flat set
    pub fn all_sensitive_paths(&self) -> HashSet<String> {
        let mut paths = HashSet::new();

        paths.extend(self.sensitive_paths.ssh.iter().cloned());
        paths.extend(self.sensitive_paths.aws.iter().cloned());
        paths.extend(self.sensitive_paths.gcp.iter().cloned());
        paths.extend(self.sensitive_paths.azure.iter().cloned());
        paths.extend(self.sensitive_paths.kubernetes.iter().cloned());
        paths.extend(self.sensitive_paths.docker.iter().cloned());
        paths.extend(self.sensitive_paths.gnupg.iter().cloned());
        paths.extend(self.sensitive_paths.keychain.iter().cloned());
        paths.extend(self.sensitive_paths.password_store.iter().cloned());
        paths.extend(self.sensitive_paths.onepassword.iter().cloned());
        paths.extend(self.sensitive_paths.macos_private.iter().cloned());
        paths.extend(self.sensitive_paths.browser_data.iter().cloned());
        paths.extend(self.sensitive_paths.credential_files.iter().cloned());
        paths.extend(self.sensitive_paths.secrets_dirs.iter().cloned());
        paths.extend(self.sensitive_paths.shell_configs.iter().cloned());
        paths.extend(self.sensitive_paths.history_files.iter().cloned());

        paths
    }

    /// Get all dangerous commands as a flat set
    pub fn all_dangerous_commands(&self) -> HashSet<String> {
        let mut commands = HashSet::new();

        commands.extend(self.dangerous_commands.file_destruction.iter().cloned());
        commands.extend(self.dangerous_commands.disk_destruction.iter().cloned());
        commands.extend(self.dangerous_commands.permission_chaos.iter().cloned());
        commands.extend(self.dangerous_commands.system_modification.iter().cloned());
        commands.extend(self.dangerous_commands.package_managers.iter().cloned());
        commands.extend(self.dangerous_commands.dangerous_file_ops.iter().cloned());
        commands.extend(self.dangerous_commands.network_exfiltration.iter().cloned());
        commands.extend(self.dangerous_commands.arbitrary_execution.iter().cloned());
        commands.extend(self.dangerous_commands.privilege_escalation.iter().cloned());

        commands
    }

    /// Get system read paths for the current platform
    pub fn system_paths_for_platform(&self) -> Vec<String> {
        let mut paths = self.system_read_paths.common.clone();

        #[cfg(target_os = "linux")]
        {
            paths.extend(self.system_read_paths.linux.libraries.iter().cloned());
            paths.extend(self.system_read_paths.linux.linker.iter().cloned());
            paths.extend(self.system_read_paths.linux.config.iter().cloned());
            paths.extend(self.system_read_paths.linux.locale.iter().cloned());
            paths.extend(self.system_read_paths.linux.ssl.iter().cloned());
            paths.extend(self.system_read_paths.linux.terminfo.iter().cloned());
            paths.extend(self.system_read_paths.linux.devices.iter().cloned());
            paths.extend(self.system_read_paths.linux.proc.iter().cloned());
            paths.extend(self.system_read_paths.linux.nix.iter().cloned());
            paths.extend(self.system_read_paths.linux.tmp.iter().cloned());
        }

        #[cfg(target_os = "macos")]
        {
            paths.extend(self.system_read_paths.macos.executables.iter().cloned());
            paths.extend(self.system_read_paths.macos.devices.iter().cloned());
            paths.extend(self.system_read_paths.macos.frameworks.iter().cloned());
            paths.extend(self.system_read_paths.macos.dyld.iter().cloned());
            paths.extend(self.system_read_paths.macos.ssl.iter().cloned());
            paths.extend(self.system_read_paths.macos.locale.iter().cloned());
            paths.extend(self.system_read_paths.macos.terminfo.iter().cloned());
            paths.extend(self.system_read_paths.macos.system.iter().cloned());
            paths.extend(self.system_read_paths.macos.user_library.iter().cloned());
            paths.extend(self.system_read_paths.macos.user_local.iter().cloned());
        }

        paths
    }

    /// Get macOS writable system paths
    #[cfg(target_os = "macos")]
    pub fn macos_writable_paths(&self) -> Vec<String> {
        self.system_read_paths.macos.writable.clone()
    }
}

/// Get all sensitive paths organized by category (for display/audit)
pub fn sensitive_paths_by_category(lists: &SecurityLists) -> HashMap<&'static str, &Vec<String>> {
    let mut categories = HashMap::new();

    categories.insert("SSH keys and config", &lists.sensitive_paths.ssh);
    categories.insert("AWS credentials", &lists.sensitive_paths.aws);
    categories.insert("GCP credentials", &lists.sensitive_paths.gcp);
    categories.insert("Azure credentials", &lists.sensitive_paths.azure);
    categories.insert("Kubernetes config", &lists.sensitive_paths.kubernetes);
    categories.insert("Docker config", &lists.sensitive_paths.docker);
    categories.insert("GnuPG keys", &lists.sensitive_paths.gnupg);
    categories.insert("macOS Keychain", &lists.sensitive_paths.keychain);
    categories.insert("Password managers", &lists.sensitive_paths.password_store);
    categories.insert("1Password", &lists.sensitive_paths.onepassword);
    categories.insert("macOS private data", &lists.sensitive_paths.macos_private);
    categories.insert("Browser data", &lists.sensitive_paths.browser_data);
    categories.insert("Credential files", &lists.sensitive_paths.credential_files);
    categories.insert("Secrets directories", &lists.sensitive_paths.secrets_dirs);
    categories.insert("Shell configurations", &lists.sensitive_paths.shell_configs);
    categories.insert("Command history", &lists.sensitive_paths.history_files);

    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lists() -> SecurityLists {
        SecurityLists {
            meta: SecurityListsMeta {
                version: 1,
                schema_version: "1.0".to_string(),
            },
            sensitive_paths: SensitivePaths {
                ssh: vec!["~/.ssh".to_string()],
                aws: vec!["~/.aws".to_string()],
                gcp: vec![],
                azure: vec![],
                kubernetes: vec![],
                docker: vec![],
                gnupg: vec![],
                keychain: vec![],
                password_store: vec![],
                onepassword: vec![],
                macos_private: vec![],
                browser_data: vec![],
                credential_files: vec![],
                secrets_dirs: vec![],
                shell_configs: vec!["~/.bashrc".to_string()],
                history_files: vec![],
            },
            dangerous_commands: DangerousCommands {
                file_destruction: vec!["rm".to_string()],
                disk_destruction: vec!["dd".to_string()],
                permission_chaos: vec![],
                system_modification: vec![],
                package_managers: vec!["pip".to_string()],
                dangerous_file_ops: vec![],
                network_exfiltration: vec![],
                arbitrary_execution: vec![],
                privilege_escalation: vec![],
            },
            system_read_paths: SystemReadPaths {
                common: vec!["/bin".to_string(), "/usr/bin".to_string()],
                linux: LinuxSystemPaths::default(),
                macos: MacosSystemPaths::default(),
            },
        }
    }

    #[test]
    fn test_all_sensitive_paths() {
        let lists = sample_lists();
        let paths = lists.all_sensitive_paths();

        assert!(paths.contains("~/.ssh"));
        assert!(paths.contains("~/.aws"));
        assert!(paths.contains("~/.bashrc"));
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_all_dangerous_commands() {
        let lists = sample_lists();
        let commands = lists.all_dangerous_commands();

        assert!(commands.contains("rm"));
        assert!(commands.contains("dd"));
        assert!(commands.contains("pip"));
        assert_eq!(commands.len(), 3);
    }

    #[test]
    fn test_system_paths_for_platform() {
        let lists = sample_lists();
        let paths = lists.system_paths_for_platform();

        assert!(paths.contains(&"/bin".to_string()));
        assert!(paths.contains(&"/usr/bin".to_string()));
    }
}
