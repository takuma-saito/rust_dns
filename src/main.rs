use std::time::Duration;
use futures_timer::Delay;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::future;

// TODO
// チェックサムの計算方法

// sudo tcpdump -n -s0 -x -X port 53
// 
// 15:52:29.862192 IP 10.230.26.56.58836 > 10.128.128.128.53: 1751+ [1au] A? google.co.jp. (41)
//         0x0000:  e0cb bc88 3b18 38f9 d363 06cf 0800 4500  ....;.8..c....E.
//         0x0010:  0045 fe35 0000 4011 cc54 {0ae6 1a38}:{10.230.26.56} {0a80 8080}:{10.128.128.128} {e5d4}:{58836} {0035}:{53} {0031}:{セグメント長（UDPのヘッダ長 + データ長）:49} {203a}:{チェックサム} {06d7 0120 0001 0000 0000 0001 0667 6f6f 676c 6502 636f 026a 7000 0001 0001 0000 2910 0000 0000 0000 00}:{データ}

// {06d7}{ID} {0120}{QR ~ RCODE, RD 1 なのでフルサービスリゾルバへの問い合わせ, AD 1 なので AD 理解できる} {0001}:{Q の数 1} {0000}:{A の数 0} {0000}:{NS の数 0} {0001}:{AR の数 1}
// {0667 6f6f 676c 65}:{6 + google}, {02 636f}:{2 + co}, {026a 70}:{2 + jp} {00}:{終了} {0001}:{タイプ1, A レコードなので} {0001}:{クラス 1, インターネットなので} {0000 2910 0000 0000 0000 00}:{謎, Additonal section のなにか}
// 

// 15:52:29.871516 IP 10.128.128.128.53 > 10.230.26.56.58836: 1751 1/0/1 A 172.217.25.67 (57)
//         0x0000:  38f9 d363 06cf e0cb bc88 3b18 0800 4500  8..c......;...E.
//         0x0010:  0055 3e43 4000 4011 4c37 0a80 8080 0ae6  .U>C@.@.L7......
//         0x0020:  1a38 0035 e5d4 0041 1875 06d7 8180 0001  .8.5...A.u......
//         0x0030:  0001 0000 0001 0667 6f6f 676c 6502 636f  .......google.co
//         0x0040:  026a 7000 0001 0001 c00c 0001 0001 0000  .jp.............
//         0x0050:  0114 0004 acd9 1943 0000 2910 0000 0000  .......C..).....
//         0x0060:  0000 00                                  ...

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

