# Event Thread + MPSC Channel

Producer thread polls crossterm events; consumer (render) thread draws; channel between.

```rust
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || loop {
    if crossterm::event::poll(Duration::from_millis(100)).unwrap_or(false)
        && let Ok(Event::Key(key_event)) = crossterm::event::read()
        && tx.send(key_event).is_err() { break; }
});
while self.running {
    terminal.draw(|frame| draw(frame, self)).expect("failed to draw");
    if let Ok(key_event) = rx.recv_timeout(Duration::from_millis(50))
        && key_event.kind == KeyEventKind::Press {
        self.handle_key(key_event.code);
    }
}
```

Shutdown implicit via dropped receiver.
