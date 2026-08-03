use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct BackendProcess {
    child: Child,
}

impl BackendProcess {
    fn spawn() -> (Self, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("should reserve a free port");
        let addr = listener
            .local_addr()
            .expect("should read the reserved address");
        drop(listener);

        let child = Command::new(env!("CARGO_BIN_EXE_dora-studio-backend"))
            .env("DORA_STUDIO_BACKEND_ADDR", addr.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("backend binary should start");

        (Self { child }, addr)
    }
}

impl Drop for BackendProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn serves_shared_model_xml() {
    let (mut backend, addr) = BackendProcess::spawn();
    wait_for_health(addr, &mut backend.child);

    let (status, body) = http_get(addr, "/models/nano_models/models/nano_full.xml")
        .expect("model asset request should succeed");

    assert_eq!(status, 200);
    assert!(body.contains("<mujoco model=\"nano_full\">"));
}

fn wait_for_health(addr: SocketAddr, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if let Some(status) = child.try_wait().expect("should check backend status") {
            panic!("backend exited before /api/health became ready with status {status}");
        }

        let last_error = match http_get(addr, "/api/health") {
            Ok((200, _)) => return,
            Ok((status, body)) => format!("unexpected health status {status}: {body}"),
            Err(error) => error.to_string(),
        };

        if Instant::now() >= deadline {
            panic!("backend did not become healthy: {last_error}");
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn http_get(addr: SocketAddr, path: &str) -> std::io::Result<(u16, String)> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(1))?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    stream.set_write_timeout(Some(Duration::from_secs(1)))?;

    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let (header, body) = response.split_once("\r\n\r\n").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing HTTP headers")
    })?;

    let status_line = header.lines().next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing HTTP status line")
    })?;

    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing HTTP status code")
        })?
        .parse::<u16>()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

    Ok((status, body.to_string()))
}
