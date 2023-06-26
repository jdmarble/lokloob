use json::object;
use std::io::Write;
use std::process::{Child, Command};
use tempfile::{NamedTempFile, TempDir};

struct TestVault {
    child: Child,
    api_port: u16,
    // Need to hold onto this until Vault finished starting.
    _config_file: NamedTempFile,
    // Need to hold onto this until Vault is killed.
    _raft_storage_path: TempDir,
}

impl TestVault {
    fn new() -> Self {
        let raft_storage_path =
            TempDir::new().expect("failed to create temporary directory for raft storage");
        let api_port = portpicker::pick_unused_port().expect("no ports free");
        let cluster_port = portpicker::pick_unused_port().expect("no ports free");

        let config_json = object! {
            "api_addr": format!("http://127.0.0.1:{}", api_port),
            "cluster_addr": format!("http://127.0.0.1:{}", cluster_port),
            "disable_mlock": true,
            "listener": { "tcp": {
                "address": format!("127.0.0.1:{}", api_port),
                "tls_disable": true
            } },
            "storage": { "raft": {
                "path": raft_storage_path.path().to_str().unwrap(),
                "node_id": "vault_0"
            } }
        };
        let mut config_file = NamedTempFile::new().expect("failed to open temporary config file");
        config_file
            .write_all(json::stringify(config_json).as_bytes())
            .expect("failed to write config file");

        let child = Command::new("vault")
            .arg("server")
            .arg(format!("-config={}", config_file.path().to_string_lossy()))
            .spawn()
            .expect("Vault failed to start");
        TestVault {
            child,
            api_port,
            _config_file: config_file,
            _raft_storage_path: raft_storage_path,
        }
    }
}

impl Drop for TestVault {
    fn drop(&mut self) {
        self.child.kill().expect("Failed to kill Vault");
        self.child.wait().expect("Failed to wait for Vault to die");
    }
}

#[test]
fn test_bootstrap() {
    let vault = TestVault::new();

    let result = Command::new("cargo")
        .args(["run", "--"])
        .arg(format!("--vault-api-port={}", vault.api_port))
        .arg("--bootstrap")
        .spawn()
        .expect("cargo run failed to start")
        .wait();

    match result {
        Ok(result_code) => assert!(result_code.success(), "Returned nonsuccess {result_code}"),
        Err(error) => panic!("cargo run failed to stop: {error}"),
    }
}

#[test]
fn test_restore() {
    let vault = TestVault::new();

    let result = Command::new("cargo")
        .args(["run", "--"])
        .arg(format!("--vault-api-port={}", vault.api_port))
        .spawn()
        .expect("cargo run failed to start")
        .wait();

    match result {
        Ok(result_code) => assert!(result_code.success(), "Returned nonsuccess {result_code}"),
        Err(error) => panic!("cargo run failed to stop: {error}"),
    }
}
