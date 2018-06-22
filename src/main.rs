extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate lru;

use actix_web::{
  client, middleware, server::HttpServer, App, AsyncResponder, Body, Error, HttpRequest,
  HttpResponse, Result,
};
use futures::{Future, Stream};
use lru::LruCache;
use std::path::PathBuf;

// struct Cache {
//   memory_cache: Cell<
// }

/// streaming client request to a streaming server response
fn streaming(req: HttpRequest) -> Result<Box<Future<Item = HttpResponse, Error = Error>>> {
  let path: PathBuf = req.match_info().query("tail")?;

  // send client request
  Ok(
    client::ClientRequest::get(format!("{},{:?}", "https://www.google.com/", path))
        .finish().unwrap()
        .send()                         // <- connect to host and send request
        .map_err(Error::from)           // <- convert SendRequestError to an Error
        .and_then(|resp| {              // <- we received client response
            Ok(HttpResponse::Ok()
               // read one chunk from client response and send this chunk to a server response
               // .from_err() converts PayloadError to an Error
               .body(Body::Streaming(Box::new(resp.from_err()))))
        })
        .responder(),
  )
}

fn main() {
  let mut cache = LruCache::new(2);
  ::std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();
  let sys = actix::System::new("rio-cache");

  HttpServer::new(|| {
    App::new()
      .middleware(middleware::Logger::default())
      .resource("/{tail:.*}", |r| r.f(streaming))
  }).workers(1)
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

  println!("Started http server: 127.0.0.1:8080");
  let _ = sys.run();
}
