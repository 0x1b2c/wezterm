use super::super::*;
use super::fixture::{client_pane, socket_path};
use promise::spawn::{set_schedulers, Runnable};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use wezterm_uds::{UnixListener, UnixStream};

/// A key as the GUI encodes it while the program in the pane has turned on
/// the kitty keyboard protocol: Ctrl+B.
const KITTY_CTRL_B: &str = "\x1b[98;5u";

/// How long a test waits for input to reach the server before failing.
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Tasks spawned for the GUI's main thread, run only when a test asks, so
/// that a test sees which work happens at once and which is deferred.
static MAIN_THREAD_TASKS: Mutex<VecDeque<Runnable>> = Mutex::new(VecDeque::new());

fn defer_main_thread_tasks() {
    set_schedulers(
        Box::new(|task| MAIN_THREAD_TASKS.lock().push_back(task)),
        Box::new(|task| MAIN_THREAD_TASKS.lock().push_back(task)),
    );
}

fn run_main_thread_tasks() {
    loop {
        let next = MAIN_THREAD_TASKS.lock().pop_front();
        match next {
            Some(task) => {
                task.run();
            }
            None => break,
        }
    }
}

/// Input for a pane as it arrives at the server.
#[derive(Debug, PartialEq)]
enum Input {
    Key(KeyCode),
    Bytes(String),
}

impl Input {
    fn from_pdu(pdu: Pdu) -> Option<Self> {
        match pdu {
            Pdu::SendKeyDown(SendKeyDown { event, .. }) => Some(Self::Key(event.key)),
            Pdu::WriteToPane(WriteToPane { data, .. }) => {
                Some(Self::Bytes(String::from_utf8(data).unwrap()))
            }
            _ => None,
        }
    }
}

/// A mux server that records the input a client sends, in the order it
/// arrives, and answers each piece of input only after `answer_delay`.
struct FakeServer {
    inputs: mpsc::Receiver<Input>,
    answered_input: Arc<AtomicBool>,
}

impl FakeServer {
    fn start(listener: UnixListener, answer_delay: Duration) -> Self {
        let (input_tx, inputs) = mpsc::channel();
        let answered_input = Arc::new(AtomicBool::new(false));
        let answered = Arc::clone(&answered_input);
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            while let Ok(decoded) = Pdu::decode(&mut stream) {
                let is_input = match Input::from_pdu(decoded.pdu) {
                    Some(input) => {
                        input_tx.send(input).ok();
                        thread::sleep(answer_delay);
                        true
                    }
                    None => false,
                };
                answer(&mut stream, decoded.serial);
                if is_input {
                    answered.store(true, Ordering::SeqCst);
                }
            }
        });
        Self {
            inputs,
            answered_input,
        }
    }

    fn has_answered_input(&self) -> bool {
        self.answered_input.load(Ordering::SeqCst)
    }

    /// Runs the GUI's deferred tasks until `count` pieces of input have
    /// reached the server, and returns them in the order they arrived.
    fn receive_inputs(&self, count: usize) -> Vec<Input> {
        let deadline = Instant::now() + DELIVERY_TIMEOUT;
        let mut inputs = vec![];
        while inputs.len() < count {
            assert!(
                Instant::now() < deadline,
                "only {:?} reached the server within {:?}",
                inputs,
                DELIVERY_TIMEOUT
            );
            run_main_thread_tasks();
            if let Ok(input) = self.inputs.recv_timeout(Duration::from_millis(10)) {
                inputs.push(input);
            }
        }
        inputs
    }
}

fn answer(stream: &mut UnixStream, serial: u64) {
    Pdu::UnitResponse(UnitResponse {})
        .encode(stream, serial)
        .unwrap();
}

/// A client pane whose server answers each piece of input after
/// `answer_delay`.
fn served_pane(name: &str, answer_delay: Duration) -> (ClientPane, FakeServer) {
    defer_main_thread_tasks();
    let path = socket_path(name);
    let server = FakeServer::start(UnixListener::bind(&path).unwrap(), answer_delay);
    (client_pane(name, &path), server)
}

#[test]
fn kitty_encoded_and_plain_keys_reach_the_server_in_the_order_typed() {
    let (pane, server) = served_pane("order", Duration::ZERO);

    pane.key_down(KeyCode::Char('a'), KeyModifiers::NONE)
        .unwrap();
    pane.writer().write_all(KITTY_CTRL_B.as_bytes()).unwrap();
    pane.key_down(KeyCode::Char('c'), KeyModifiers::NONE)
        .unwrap();

    assert_eq!(
        server.receive_inputs(3),
        vec![
            Input::Key(KeyCode::Char('a')),
            Input::Bytes(KITTY_CTRL_B.to_string()),
            Input::Key(KeyCode::Char('c')),
        ]
    );
}

#[test]
fn writing_kitty_encoded_keys_to_a_mux_pane_does_not_wait_for_the_server() {
    // Long enough that a write waiting for the answer is caught, short
    // enough that such a write fails the test instead of hanging it.
    let (pane, server) = served_pane("no-wait", Duration::from_secs(2));

    pane.writer().write_all(KITTY_CTRL_B.as_bytes()).unwrap();

    assert!(
        !server.has_answered_input(),
        "the write returned only after the server answered"
    );
    assert_eq!(
        server.receive_inputs(1),
        vec![Input::Bytes(KITTY_CTRL_B.to_string())]
    );
}

#[test]
fn a_key_release_that_encodes_to_nothing_sends_nothing() {
    let (pane, server) = served_pane("empty-release", Duration::ZERO);

    pane.writer().write_all(b"").unwrap();
    pane.key_down(KeyCode::Char('a'), KeyModifiers::NONE)
        .unwrap();

    assert_eq!(
        server.receive_inputs(1),
        vec![Input::Key(KeyCode::Char('a'))]
    );
}

#[test]
fn an_ime_commit_after_a_kitty_key_arrives_after_it() {
    let (pane, server) = served_pane("ime", Duration::ZERO);

    pane.writer().write_all(KITTY_CTRL_B.as_bytes()).unwrap();
    pane.send_composed_text("日本").unwrap();

    assert_eq!(
        server.receive_inputs(2),
        vec![
            Input::Bytes(KITTY_CTRL_B.to_string()),
            Input::Bytes("日本".to_string()),
        ]
    );
}
