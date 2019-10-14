use std::time::Duration;
use futures_timer::Delay;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::future;

async fn job((i, u): &(usize, usize)) -> usize {
  println!("begin {:?} => {:?}", i, u);
  Delay::new(Duration::from_secs(*u as u64)).await;
  println!("end {:?} => {:?}", i, u);
  *i
}

async fn executor(i: usize, requests: &[(usize, usize)]) -> Vec<usize> {
  let mut vec = vec![];
  println!("id: {:?}", i);
  for request in requests.iter() {
    vec.push(job(request).await)
  }
  vec
}

async fn run() {
  let mut durations = vec![];
  let mut futures = vec![];
  for i in 0..30 {
    durations.push((i, rand::random::<usize>() % 10));
  }
  let Pool = 10;
  let requests_chunk = durations.chunks(durations.len() / Pool);
  for (i, requests) in requests_chunk.enumerate() {
    futures.push(executor(i, requests).boxed());
  }
  future::join_all(futures).await;
}

fn main() -> () {
  block_on(run());
}

