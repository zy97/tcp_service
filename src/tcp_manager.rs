use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use deadpool::managed::{self, RecycleError};
use tracing::{info, trace};

pub mod tcp_manager {}
#[derive(Clone)]
pub struct ModbusContext {
    pub addr: String,
}
#[derive(Debug)]
pub enum Error {
    Fail,
}
impl managed::Manager for ModbusContext {
    type Type = TcpStream;
    type Error = Error;

    async fn create(&self) -> Result<TcpStream, Error> {
        let socket_addr = self.addr.parse::<SocketAddr>().unwrap();
        match TcpStream::connect(socket_addr) {
            Ok(conn) => {
                trace!("连接到{}", self.addr);
                conn.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
                Ok(conn)
            }
            _ => {
                info!("不能连接到{}", self.addr);
                Err(Error::Fail)
            }
        }
    }

    async fn recycle(
        &self,
        conn: &mut TcpStream,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Error> {
        match is_connection_alive(conn) {
            true => Ok(()),
            _ => Err(RecycleError::Message(std::borrow::Cow::Borrowed(
                "can't recycle",
            ))),
        }
    }
    fn detach(&self, _obj: &mut Self::Type) {}
}
impl ModbusContext {}
fn is_connection_alive(stream: &mut TcpStream) -> bool {
    // 尝试发送一个探测数据
    match stream.write(&[0]) {
        Ok(_) => {
            // 尝试读取一些数据作为额外检查
            let mut buffer = [0; 1];
            match stream.read(&mut buffer) {
                Ok(0) => false,                                          // 连接已关闭
                Ok(_) => true,                                           // 连接正常
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => true, // 暂时没有数据
                Err(_) => false,                                         // 其他错误
            }
        }
        Err(ref e) if e.kind() == ErrorKind::BrokenPipe => false, // 连接已断开
        Err(_) => false,                                          // 其他发送错误
    }
}
