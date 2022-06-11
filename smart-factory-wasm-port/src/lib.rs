extern crate wasm_bindgen;

use js_sys::Promise;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use smart_factory_environment::empty_environment::{
    EmptyEnvironmentSettings, InfiniteEmptyEnvironment,
};
use smart_factory_environment::environment::AgentEnvironment;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(closure: JsValue, millis: f64) -> JsValue;

    #[wasm_bindgen(js_name = clearTimeout)]
    fn clear_timeout(id: &JsValue);
}

pub struct Timeout {
    id: JsValue,
    inner: JsFuture,
}

impl Timeout {
    pub fn new(dur: Duration) -> Timeout {
        let millis = dur
            .as_secs()
            .checked_mul(1000)
            .unwrap()
            .checked_add(dur.subsec_millis() as u64)
            .unwrap() as f64; // TODO: checked cast

        let mut id = None;
        let promise = Promise::new(&mut |resolve, _reject| {
            id = Some(set_timeout(resolve.into(), millis));
        });

        Timeout {
            id: id.unwrap(),
            inner: JsFuture::from(promise),
        }
    }
}

impl Future for Timeout {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        Pin::new(&mut self.inner).poll(cx).map(|_| ())
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        clear_timeout(&self.id);
    }
}

pub async fn sleep(duration: Duration) {
    Timeout::new(duration).await;
}

pub fn run_empty() {
    const ITER_COUNT_SLEEP: u64 = 5000;
    const SLEEP_DURATION_MS: u64 = 100;

    let mut env = InfiniteEmptyEnvironment::new(log, sleep);
    env.run(EmptyEnvironmentSettings::new(
        1,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
        u64::MAX
    ));
}
