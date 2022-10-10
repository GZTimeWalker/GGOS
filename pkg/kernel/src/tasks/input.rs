use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use pc_keyboard::DecodedKey;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker
};

once_mutex!(pub INPUT_QUEUE: ArrayQueue<DecodedKey>);

const DEFAULT_BUF_SIZE: usize = 128;

guard_access_fn!(pub get_input_queue(INPUT_QUEUE: ArrayQueue<DecodedKey>));

static INPUT_WAKER: AtomicWaker = AtomicWaker::new();

pub fn push_key(key: DecodedKey) {
    if let Some(queue) = get_input_queue() {
        if queue.push(key).is_ok() {
            INPUT_WAKER.wake()
        }
    }
}

pub struct InputStream;

impl InputStream {
    pub fn new() -> Self {
        init_INPUT_QUEUE(ArrayQueue::new(DEFAULT_BUF_SIZE));
        info!("Input stream Initialized.");
        Self
    }
}

impl Stream for InputStream {
    type Item = DecodedKey;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = get_input_queue_for_sure();
        if let Some(key) = queue.pop() {
            Poll::Ready(Some(key))
        } else {
            INPUT_WAKER.register(&cx.waker());
            match queue.pop() {
                Some(key) => {
                    INPUT_WAKER.take();
                    Poll::Ready(Some(key))
                }
                None => Poll::Pending,
            }
        }
    }
}

pub async fn get_key() {
    let mut input = InputStream::new();
    while let Some(key) = input.next().await {
        match key {
            DecodedKey::Unicode(c) => print!("{}", c),
            DecodedKey::RawKey(k) => print!("{:?}", k),
        }
    }
}
