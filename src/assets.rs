use std::future::{poll_fn, Future};
use std::task::Poll;

use macroquad::prelude::*;

use crate::art::Sprites;
use crate::data::{Data, Lang};

pub struct Assets {
    pub font: Font,
    pub stamp: Texture2D,
    pub art: Sprites,
    pub data: Data,
}

impl Assets {
    pub async fn load(lang: Lang) -> Assets {
        let font = load_ttf_font("assets/fonts/PixelOperator.ttf")
            .await
            .expect("font");
        let stamp = load_texture("assets/sprites/stamp.png")
            .await
            .expect("stamp");
        stamp.set_filter(FilterMode::Nearest);
        Assets {
            font,
            stamp,
            art: Sprites::load().await,
            data: Data::load(lang).await,
        }
    }
}

/// Await every future at once. On the web each asset load is a fetch, and
/// awaiting them one by one costs a network round trip per file.
pub async fn join_all<F: Future>(futures: impl IntoIterator<Item = F>) -> Vec<F::Output> {
    let mut futures: Vec<_> = futures.into_iter().map(Box::pin).collect();
    let mut out: Vec<Option<F::Output>> = futures.iter().map(|_| None).collect();
    poll_fn(|cx| {
        let mut done = true;
        for (future, slot) in futures.iter_mut().zip(&mut out) {
            if slot.is_none() {
                match future.as_mut().poll(cx) {
                    Poll::Ready(v) => *slot = Some(v),
                    Poll::Pending => done = false,
                }
            }
        }
        if done {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    out.into_iter().map(Option::unwrap).collect()
}
