use std::{
  io::ErrorKind,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use car_utils::{
  buffer::RingBuffer, command::Command, Response as Statistics, ResponseType, REQUEST_HEADER_LEN, RESPONSE_HEADER_LEN,
};
use num_traits::FromPrimitive;
use tauri::{Emitter, Error, Listener};
use tokio::{net::TcpStream, sync::Notify};

/// TODO: 字节序
/// TODO: 客户端卡死问题 ？
/// 连接
#[tauri::command]
pub async fn connect(window: tauri::Window, addr: &str) -> Result<(), Error> {
  println!("connecting to {}", addr);
  let stream = TcpStream::connect(addr)
    .await
    .inspect_err(|err| {
      let _ = window.emit(
        "connect-client", //
        serde_json::to_value(&format!("{{ \"status\": false, \"msg\": \"{:}\"}}", err)).unwrap(),
      );
    })
    .map_err(Error::Io)?;
  let stream = Arc::new(stream);
  let should_shutdown = Arc::new(AtomicBool::new(false));
  let (tx_notify, tx_buffer) = (Arc::new(Notify::new()), Arc::new(std::sync::Mutex::new(RingBuffer::<1024>::new())));

  let mut listen_ids = Vec::new();
  listen_ids.push(window.listen("close-server", {
    let shutdown = Arc::clone(&should_shutdown);
    move |_event| {
      println!("close event");
      shutdown.store(true, Ordering::Release);
    }
  }));
  listen_ids.push(window.listen("command-server", {
    let (tx_notify, tx_buffer) = (Arc::clone(&tx_notify), Arc::clone(&tx_buffer));

    move |event| match serde_json::from_str::<Command>(event.payload()) {
      Ok(command) => {
        let mut tx_buffer = tx_buffer.lock().unwrap();
        if tx_buffer.free_len() < command.buf_len() + REQUEST_HEADER_LEN {
          println!("丢弃 {:?}", command);
          return;
        }
        tx_buffer.enqueue_one(command.buf_len() as u8).unwrap();
        // TODO: refactor
        let mut buf = vec![0; command.buf_len()];
        command.write(&mut buf);
        buf.into_iter().for_each(|x| {
          tx_buffer.enqueue_one(x).unwrap();
        });

        tx_notify.notify_one();
      }
      Err(_) => {
        println!("未知命令");
      }
    }
  }));

  let _ = window.emit(
    "connect-client", //
    serde_json::to_value(&format!("{{\"status\":true, \"addr\": \"{}\"}}", stream.peer_addr().unwrap())).unwrap(),
  ); // 触发连接成功事件

  // 写任务
  let write_task = tokio::spawn({
    let should_shutdown = Arc::clone(&should_shutdown);
    let stream = Arc::clone(&stream);
    let tx_buffer = Arc::clone(&tx_buffer);
    let tx_notify = Arc::clone(&tx_notify);

    async move {
      loop {
        tx_notify.notified().await;

        while !should_shutdown.load(Ordering::Acquire) {
          let _ = stream.writable().await;

          let mut tx_buffer = tx_buffer.lock().unwrap();
          let buffer_len = tx_buffer.len();
          tx_buffer.dequeue_with(buffer_len, |buf| stream.try_write(buf).unwrap_or(0));
        }
      }
    }
  });

  // 接受数据并发送给前端
  let mut rx_buffer = RingBuffer::<1024>::new();
  while !should_shutdown.load(Ordering::Acquire) {
    stream.readable().await.ok();

    let _ = rx_buffer.enqueue_with(|buf| {
      match stream.try_read(buf) {
        Ok(0) => {
          window.emit("close-client", "").unwrap();

          should_shutdown.store(true, Ordering::Release); // 关闭连接
          (0, 0)
        }
        Ok(size) => (size, size), // 后续处理
        Err(err) => {
          match err.kind() {
            ErrorKind::WouldBlock => {}
            _ => {
              println!("error: {}, {}", err.kind(), err)
            }
          }
          (0, 0)
        }
      }
    });

    while rx_buffer.peek_one().map_or(false, |len| rx_buffer.len() >= len as usize + RESPONSE_HEADER_LEN) {
      let len = rx_buffer.dequeue_one().unwrap() as usize; // 负载长度
      let response_type = rx_buffer.dequeue_one().unwrap();

      rx_buffer.congestion_alloced(len);
      rx_buffer.dequeue_with(len, |payload| {
        //
        match ResponseType::from_u8(response_type) {
          Some(resp) => match resp {
            ResponseType::Statistics => {
              let statistics = serde_json::from_slice::<Statistics>(payload).unwrap();
              println!("statistics: {:?}", statistics);
              let _ = window.emit("statistics", statistics).inspect_err(|e| {
                println!("{:?}", e);
              });

              // let _ = statistics_channel.send(statistics).inspect_err(|e| {
              //   println!("{:?}", e);
              // });
            }
          },
          None => {
            println!("unknown response");
          }
        }
      });
    }
  }

  // 解除监听
  listen_ids.into_iter().for_each(|id| {
    window.unlisten(id);
  });

  write_task.await?;
  println!("连接关闭: {:}", stream.peer_addr().unwrap());

  Ok(())
}
