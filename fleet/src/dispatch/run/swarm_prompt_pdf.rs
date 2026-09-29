//! Parent side of PDF intake: the file's bytes go to `fleet __pdf-text` (`pdf_text_cmd.rs`), a
//! throwaway child with a heap cap, and come back as text. This side bounds the rest: a
//! wall-clock deadline after which the child is killed, and bounded reads of its output, so no
//! PDF can make the CLI itself spend unbounded memory or time.

use super::swarm_prompt_file::{Failure, MAX_PROMPT_BYTES};
use std::io::{Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(30);

pub(super) fn text(bytes: &[u8]) -> Result<String, Failure> {
    let env = |what: &str, e: std::io::Error| Failure::Env(format!("{what} the PDF parser: {e}"));
    let exe = std::env::current_exe().map_err(|e| env("cannot locate", e))?;
    let mut child = Command::new(exe)
        .arg("__pdf-text")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| env("cannot start", e))?;
    let (mut stdin, input) = (child.stdin.take(), bytes.to_vec());
    // A child that dies early closes the pipe; the failed write is then moot, not an error.
    let feeder = spawn(move || stdin.as_mut().map(|pipe| pipe.write_all(&input)));
    let stdout = drain(child.stdout.take(), MAX_PROMPT_BYTES + 1);
    let stderr = drain(child.stderr.take(), 64 * 1024);
    let status = wait(&mut child).map_err(|e| env("lost", e))?;
    let _ = feeder.join();
    let (out, err) = (
        stdout.join().unwrap_or_default(),
        stderr.join().unwrap_or_default(),
    );
    super::swarm_prompt_pdf_verdict::verdict(status, DEADLINE, out, &String::from_utf8_lossy(&err))
}

/// Keep at most `cap` bytes, then discard the rest, so a chatty child can never block on a full
/// pipe (and so ride out the deadline) while this side holds only a bounded buffer.
fn drain<R: Read + Send + 'static>(pipe: Option<R>, cap: usize) -> JoinHandle<Vec<u8>> {
    spawn(move || {
        let mut kept = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.by_ref().take(cap as u64).read_to_end(&mut kept);
            let _ = std::io::copy(&mut pipe, &mut std::io::sink());
        }
        kept
    })
}

/// `None` once the deadline passes -- the child is then killed and reaped, never left behind.
fn wait(child: &mut Child) -> std::io::Result<Option<ExitStatus>> {
    let start = Instant::now();
    while start.elapsed() < DEADLINE {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = child.kill();
    child.wait().map(|_| None)
}
