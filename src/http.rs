use poll_promise::{Promise, Sender};
use reqwest::blocking::{RequestBuilder, Response};

pub fn try_http<T: Send>(
    mut request: RequestBuilder,
    callback: fn(Response) -> T,
) -> Option<Promise<T>> {
    // I promise <T> but how to error handle?
    // stored result: Option<Promise<T>>
    let (sender, promise): (Sender<T>, Promise<T>) = Promise::new();
    std::thread::spawn(move || {
        let response = request.send().unwrap(); // don't. maybe wrap T in option and bypass sending back
        let result = callback(response);
        sender.send(result);
    });
    Some(promise)
}
