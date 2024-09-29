use std::result::Result;

#[derive(Debug)]
pub enum Error {
  Empty,
  Full,
}

pub struct RingBuffer<const N: usize = 1024> {
  buf: [u8; N],
  pub offset: usize,
  len: usize,
}

impl<const N: usize> RingBuffer<N> {
  pub const CAPACITY: usize = N;

  pub fn new() -> Self {
    Self { buf: [0; N], offset: 0, len: 0 }
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    N
  }

  #[inline]
  pub fn free_len(&self) -> usize {
    self.capacity() - self.len
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_full(&self) -> bool {
    self.len() == N
  }

  /// 保证前 len 字节连续
  pub fn congestion_alloced(&mut self, len: usize) {
    if self.offset + len > self.capacity() {
      let diff = self.get_index(len) + 1;
      self.buf.rotate_left(diff);
      self.offset -= diff;
    }
  }

  pub fn peek_one(&self) -> Option<u8> {
    if self.is_empty() {
      None
    } else {
      Some(self.buf[self.offset])
    }
  }

  pub fn dequeue_one(&mut self) -> Option<u8> {
    if self.is_empty() {
      None
    } else {
      let res = self.buf[self.offset];

      self.offset = self.get_index(1);
      self.len -= 1;
      Some(res)
    }
  }

  /// 出队 最大max_len个元素
  pub fn dequeue_with<R>(&mut self, max_len: usize, mut emit: impl FnMut(&[u8]) -> R) -> R {
    let max_len = usize::min(max_len, self.len);
    let len = if self.offset + max_len <= self.capacity() { max_len } else { self.capacity() - self.offset };
    let result = emit(&self.buf[self.offset..(self.offset + len)]);

    self.offset = self.get_index(len);
    self.len -= len;

    result
  }

  /// 填充
  pub fn enqueue_one(&mut self, value: u8) -> Result<(), Error> {
    if self.is_full() {
      Err(Error::Full)
    } else {
      self.buf[self.get_index(self.len)] = value;
      self.len += 1;

      Ok(())
    }
  }

  /// 填充
  pub fn enqueue_with<R>(&mut self, mut emit: impl FnMut(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
    if self.is_empty() {
      self.offset = 0;
    }

    if self.is_full() {
      Err(Error::Full)
    } else {
      //  ...end    ... begin...
      //  ...begin .... end ...
      let begin = self.get_index(self.len);
      let end = if self.offset + self.len <= self.capacity() {
        self.capacity() //
      } else {
        self.offset
      };

      let (len, result) = emit(&mut self.buf[begin..end]);
      self.len += len;
      Ok(result)
    }
  }

  fn get_index(&self, index: usize) -> usize {
    (self.offset + index) % self.capacity()
  }
}

impl<const N: usize> Default for RingBuffer<N> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod test {
  use super::RingBuffer;

  #[test]
  fn test() {
    let mut buffer = RingBuffer::<13>::new();
    assert_eq!(buffer.dequeue_one(), None);

    assert!(buffer.enqueue_one(12).is_ok());
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer.peek_one(), Some(12));

    assert!(buffer
      .enqueue_with(|buf| {
        let len = buf.len();
        buf.copy_from_slice(&[2].repeat(len));

        // buf[0..12].copy_from_slice(&[1].repeat(12));
        (len, ())
      })
      .is_ok());
    assert_eq!(buffer.len(), 13);

    assert_eq!(buffer.dequeue_one(), Some(12));
    assert!(buffer.enqueue_one(13).is_ok());
    assert!(buffer.enqueue_one(13).is_err());

    buffer.dequeue_with(buffer.len(), |buf| {
      assert_eq!(buf.len(), 12);
    });
  }

  #[test]
  fn test_enqueue() {
    let mut buffer = RingBuffer::<0>::new();
    assert!(buffer.enqueue_one(1).is_err());

    let mut buffer = RingBuffer::<1>::new();
    assert!(buffer.enqueue_one(1).is_ok());
  }

  #[test]
  fn test_dequeue() {
    let mut buffer = RingBuffer::<3>::new();
    buffer.offset = 2;
    buffer.len = 2;
    buffer.dequeue_with(buffer.len(), |buf| {
      assert_eq!(buf.len(), 1);
    });

    assert_eq!(buffer.offset, 0);
    buffer.dequeue_with(buffer.len(), |buf| {
      assert_eq!(buf.len(), 1);
    });
  }

  #[test]
  fn test_congestion() {
    let mut buffer = RingBuffer::<10>::new();
    buffer.offset = 8;
    buffer.len = 7;

    buffer.buf[buffer.offset] = 1;
    buffer.buf[buffer.get_index(buffer.len)] = 2;

    buffer.congestion_alloced(buffer.len);

    println!("{} {}", buffer.offset, buffer.len)
  }
}
