use std::{thread, time};
use std::time::Duration;
use futures_timer::Delay;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::future;
use rand::prelude::*;

async fn send((i, u): (usize, usize)) -> usize {
  println!("begin {:?} => {:?}", i, u);
  Delay::new(Duration::from_secs(u as u64)).await;
  println!("end {:?} => {:?}", i, u);
  i
}

async fn run() {
  let mut durations = vec![];
  let mut futures = vec![];
  let mut v = &0 as *const i32 as usize;
  for i in 0..30 {
    v = rand::random::<usize>() % 10;
    durations.push((i, v));
  }
  while durations.len() > 0 {
    // if futures.len() > 3 {
    //   future::select_all(futures);
    // } else {
      futures.push(send(durations.pop().unwrap()).boxed());
    // }
  }
  future::join_all(futures).await;
}

fn main() -> () {
  block_on(run());
}

