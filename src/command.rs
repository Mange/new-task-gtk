use std::process;
use std::process::{Command, Child, ExitStatus};
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Receiver};
use std::io::{BufRead, BufReader};
use std::thread;

pub struct TaskWarrior {}

pub struct CommandStream {
    stdout_thread: Option<JoinHandle<Result<ExitStatus, ::std::io::Error>>>,
    receiver: Receiver<String>,
    status: Option<ExitStatus>,
}

pub enum StreamStatus {
    Line(String),
    Wait,
    Complete,
    Failed(i32),
    Error(String),
}

impl TaskWarrior {
    pub fn add(text: &str) -> Result<CommandStream, String> {
        let shell = ::std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        let child = Command::new(shell)
            .arg("-c")
            .arg(format!("task add {} 2>&1", text))
            .stdout(process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Could not spawn process: {}", e))?;

        Ok(CommandStream::new(child))
    }
}

impl CommandStream {
    fn new(mut child: Child) -> Self {
        let (sender, receiver) = channel();

        let stdout = child.stdout.take().expect(
            "Provided command did not have stdout as a pipe",
        );
        let stdout_thread = {
            thread::spawn(move || {
                for line in BufReader::new(stdout).lines().flat_map(Result::ok) {
                    if let Err(_) = sender.send(line) {
                        break;
                    }
                }
                drop(sender);
                child.wait()
            })
        };

        CommandStream {
            status: None,
            stdout_thread: Some(stdout_thread),
            receiver: receiver,
        }
    }

    pub fn try_next_line(&mut self) -> StreamStatus {
        use std::sync::mpsc::TryRecvError::*;

        match self.receiver.try_recv() {
            Ok(line) => return StreamStatus::Line(line),
            Err(Empty) => return StreamStatus::Wait,
            Err(Disconnected) => {}
        }

        self.reap();
        if let Some(ref status) = self.status {
            StreamStatus::from_status(status)
        } else {
            StreamStatus::Error("Cannot determine exit status".to_owned())
        }
    }

    fn reap(&mut self) {
        if let Some(thread) = self.stdout_thread.take() {
            if let Ok(Ok(exit_status)) = thread.join() {
                self.status = Some(exit_status)
            }
        }
    }
}

impl StreamStatus {
    fn from_status(status: &ExitStatus) -> StreamStatus {
        if status.success() {
            StreamStatus::Complete
        } else {
            StreamStatus::Failed(status.code().unwrap_or(1))
        }
    }
}
