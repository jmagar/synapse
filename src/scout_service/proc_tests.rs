//! Unit tests for scout process/disk operations (ps/df).

use crate::synapse::HostConfig;

#[test]
fn ps_rejects_invalid_sort() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let host = HostConfig::local();

    use crate::ssh::{CommandOutput, SshExecutor};
    use async_trait::async_trait;
    struct NoopExec;
    #[async_trait]
    impl SshExecutor for NoopExec {
        async fn exec(&self, _: &HostConfig, _: &str, _: &[&str]) -> anyhow::Result<CommandOutput> {
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
            })
        }
    }

    let result = rt.block_on(super::ps(
        &host,
        &NoopExec,
        Some("inject; rm -rf /"),
        None,
        None,
        None,
    ));
    assert!(result.is_err(), "invalid sort must be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("invalid sort"), "{msg}");
}

#[test]
fn df_rejects_relative_path() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let host = HostConfig::local();

    use crate::ssh::{CommandOutput, SshExecutor};
    use async_trait::async_trait;
    struct NoopExec;
    #[async_trait]
    impl SshExecutor for NoopExec {
        async fn exec(&self, _: &HostConfig, _: &str, _: &[&str]) -> anyhow::Result<CommandOutput> {
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: Some(0),
            })
        }
    }

    let result = rt.block_on(super::df(&host, &NoopExec, Some("relative/path")));
    assert!(result.is_err(), "relative path must be rejected");
}
