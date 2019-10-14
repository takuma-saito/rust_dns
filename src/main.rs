use std::time::Duration;
use futures_timer::Delay;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::future;
use std::slice;
use tokio::net::UdpSocket;
use std::net::SocketAddr;

// TODO
// チェックサムの計算方法

// sudo tcpdump -n -s0 -x -X port 53
// 
// 15:52:29.862192 IP 10.230.26.56.58836 > 10.128.128.128.53: 1751+ [1au] A? google.co.jp. (41)
//         0x0000:  e0cb bc88 3b18 38f9 d363 06cf 0800 4500  ....;.8..c....E.
//         0x0010:  0045 fe35 0000 4011 cc54 {0ae6 1a38}:{10.230.26.56} {0a80 8080}:{10.128.128.128} {e5d4}:{58836} {0035}:{53} {0031}:{セグメント長（UDPのヘッダ長 + データ長）:49} {203a}:{チェックサム} {06d7 0120 0001 0000 0000 0001 0667 6f6f 676c 6502 636f 026a 7000 0001 0001 0000 2910 0000 0000 0000 00}:{データ}

// {06d7}{ID} {0120}{QR ~ RCODE, RD 1 なのでフルサービスリゾルバへの問い合わせ, AD 1 なので AD 理解できる} {0001}:{Q の数 1} {0000}:{A の数 0} {0000}:{NS の数 0} {0001}:{AR の数 1}
// {0667 6f6f 676c 65}:{6 + google}, {02 636f}:{2 + co}, {026a 70}:{2 + jp} {00}:{終了} {0001}:{タイプ1, A レコードなので} {0001}:{クラス 1, インターネットなので} {0000 2910 0000 0000 0000 00}:{謎, Additonal section のなにか}

// 15:52:29.871516 IP 10.128.128.128.53 > 10.230.26.56.58836: 1751 1/0/1 A 172.217.25.67 (57)
//         0x0000:  38f9 d363 06cf e0cb bc88 3b18 0800 4500  8..c......;...E.
//         0x0010:  0055 3e43 4000 4011 4c37 0a80 8080 0ae6  .U>C@.@.L7......
//         0x0020:  1a38 0035 e5d4 0041 1875 06d7 8180 0001  .8.5...A.u......
//         0x0030:  0001 0000 0001 0667 6f6f 676c 6502 636f  .......google.co
//         0x0040:  026a 7000 0001 0001 c00c 0001 0001 0000  .jp.............
//         0x0050:  0114 0004 acd9 1943 0000 2910 0000 0000  .......C..).....
//         0x0060:  0000 00                                  ...

#[repr(C)]
struct DnsResolver {
  id: u16,
  qr: u16,
  cq: u16,
  ca: u16,
  cns: u16,
  cad: u16,
  t: u16,
  class: u16,
  query: Vec<u8>,
}

impl DnsResolver {
  fn new(domain: String) -> Self {
    let mut query = domain.into_bytes(); // .google.co.jp.
    let mut pin = 0;
    let mut count = 1;
    while (pin + count) < query.len() {
      if query[pin + count] == 0x2e {
        query[pin + count] = 0;
        query[pin] = (count-1) as u8;
        pin += count;
        count = 1;
      } else {
        count += 1;
      }
      if (count > 255) { panic!("name length > 255"); }
    }
    query[pin + count - 1] = 0;
    Self {
      id: rand::random::<u16>(),
      qr: 0x120,
      cq: 0x1,
      ca: 0x0,
      cns: 0x0,
      cad: 0x0,
      query: query,
      t: 0x1,
      class: 0x1,
    }
  }
  fn as_u8(&self) -> Vec<u8> {
    let mut ptr = (self as *const Self) as *const u16;
    let mut query = self.query.to_owned();
    let mut vec = vec![];
    let slice: &[u16] = unsafe { slice::from_raw_parts(ptr, 8) }; // TODO
    for (i, u) in slice.iter().enumerate() {
      if i == 6 {
        vec.append(&mut query);
      }
      let v = *u;
      vec.push((v >> 8) as u8);
      vec.push((v & 0xff) as u8);
    }
    vec
  }
  fn hex(&self) -> String {
    self.as_u8().iter().map(|x| { format!("{:02x}", x) }).collect::<String>()
  }
}

async fn job((i, u): &(usize, usize)) -> usize {
  println!("begin {:?} => {:?}", i, u);
  Delay::new(Duration::from_millis(((*u) * 100) as u64)).await;
  println!("end {:?} => {:?}", i, u);
  *i
}

async fn dispatch(i: usize, requests: &[(usize, usize)]) -> Vec<usize> {
  let mut vec = vec![];
  for request in requests.iter() {
    vec.push(job(request).await)
  }
  vec
}

async fn run() {
  let mut durations = vec![];
  let mut futures = vec![];
  for i in 0..100 {
    durations.push((i, rand::random::<usize>() % 67));
  }
  let Pool = 30;
  let requests_chunk = durations.chunks(durations.len() / Pool);
  for (i, requests) in requests_chunk.enumerate() {
    futures.push(dispatch(i, requests).boxed());
  }
  future::join_all(futures).await;
}

async fn udp_client() {  
  let remote_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
  let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
  let mut socket = UdpSocket::bind(local_addr).await.unwrap();
}

#[tokio::main]
async fn main() -> () {
  //block_on(run());
  // let v = DnsResolver::new(".google.co.jp.".to_string());
  // println!("{}", v.hex());
  udp_client();
}
