use std::process::{Child, Command, Stdio};
use tempfile::NamedTempFile;

pub struct AnvilInstance {
    process: Child,
    port: u16,
    temp_file: NamedTempFile,
}

impl AnvilInstance {
    pub fn fork(rpc_url: &str, fork_block: Option<u64>) -> Result<Self, Box<dyn std::error::Error>> {
        let port = find_free_port()?;
        let temp_file = NamedTempFile::new()?;

        let mut cmd = Command::new("anvil");
        cmd
            .arg("--fork-url")
            .arg(rpc_url)
            .arg("--port")
            .arg(port.to_string())
            .arg("--fork-keep-alive")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some(block) = fork_block {
            cmd.arg("--fork-block-number").arg(block.to_string());
        }

        let process = cmd.spawn()?;

        // Attendre qu'Anvil soit prêt
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(Self { process, port, temp_file })
    }

    pub fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn ws_endpoint(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    pub fn anvil_rpc(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("anvil")
            .arg("--rpc-url")
            .arg(&self.endpoint())
            .args(args)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Drop for AnvilInstance {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

fn find_free_port() -> Result<u16, Box<dyn std::error::Error>> {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}