use crate::application::session::{BrowserEvent, BrowserMessage};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::{SendError, Sender},
};
#[derive(Clone)]
pub(super) struct EventSink {
    sender: Sender<BrowserMessage>,
    generation: Rc<Cell<u64>>,
    frozen: Option<u64>,
    document: Rc<RefCell<String>>,
}
impl EventSink {
    pub fn new(
        sender: Sender<BrowserMessage>,
        generation: Rc<Cell<u64>>,
        document: Rc<RefCell<String>>,
    ) -> Self {
        Self {
            sender,
            generation,
            frozen: None,
            document,
        }
    }
    pub fn freeze(&self) -> Self {
        Self {
            frozen: Some(self.generation.get()),
            ..self.clone()
        }
    }
    pub fn send(&self, event: BrowserEvent) -> Result<(), SendError<BrowserMessage>> {
        if matches!(event, BrowserEvent::Media(_))
            && !crate::domain::navigation::player_page(&self.document.borrow())
        {
            return Ok(());
        }
        self.sender.send(BrowserMessage {
            generation: self.frozen.unwrap_or_else(|| self.generation.get()),
            event,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_media_on_catalogue_is_discarded() {
        let (tx, rx) = std::sync::mpsc::channel();
        let events = EventSink::new(
            tx,
            Rc::new(Cell::new(7)),
            Rc::new(RefCell::new("https://catalog.example.test/종영".into())),
        );
        events
            .send(BrowserEvent::Media(crate::domain::media::MediaSource {
                url: "https://cdn.example/old.m3u8".into(),
                referer: "https://media.example.test/".into(),
                hls: true,
            }))
            .unwrap();
        assert!(rx.try_recv().is_err());
    }
    #[test]
    fn asynchronous_callback_keeps_its_original_document_generation() {
        let (tx, rx) = std::sync::mpsc::channel();
        let generation = Rc::new(Cell::new(3));
        let events = EventSink::new(
            tx,
            generation.clone(),
            Rc::new(RefCell::new(
                "https://frame.example.test/tv-show.php?v=1".into(),
            )),
        );
        let callback = events.freeze();
        generation.set(4);
        callback.send(BrowserEvent::Loaded).unwrap();
        events.send(BrowserEvent::Loaded).unwrap();
        assert_eq!(rx.recv().unwrap().generation, 3);
        assert_eq!(rx.recv().unwrap().generation, 4);
    }
}
