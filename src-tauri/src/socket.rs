use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn bind(port: u16) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        Self { listener }
    }

    pub async fn accept(
        &self,
        ctrl_msg_recv: tokio::sync::broadcast::Receiver<String>,
        device_reply_send: tokio::sync::mpsc::Sender<String>,
    ) {
        let (client, _) = self.listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (read_half, write_half) = client.into_split();

            tokio::spawn(async move {
                read_socket(read_half, device_reply_send).await;
            });

            tokio::spawn(async move {
                recv_app_msg(write_half, ctrl_msg_recv).await;
            });
        });
    }
}

async fn recv_app_msg(
    mut write_half: OwnedWriteHalf,
    mut ctrl_msg_recv: tokio::sync::broadcast::Receiver<String>,
) {
    loop {
        ctrl_msg_recv.recv().await.unwrap();
        write_half.write_all(b"hello from server").await.unwrap();
    }
}

/// 从客户端读取
async fn read_socket(reader: OwnedReadHalf, device_reply_send: tokio::sync::mpsc::Sender<String>) {
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buf = String::new();
    loop {
        match buf_reader.read_line(&mut buf).await {
            Err(_e) => {
                eprintln!("read from client error");
                break;
            }
            // 遇到了EOF
            Ok(0) => {
                println!("client closed");
                break;
            }
            Ok(n) => {
                // read_line()读取时会包含换行符，因此去除行尾换行符
                // 将buf.drain(。。)会将buf清空，下一次read_line读取的内容将从头填充而不是追加
                buf.pop();
                let content = buf.drain(..).as_str().to_string();
                println!("read {} bytes from client. content: {}", n, content);
                device_reply_send.send(content).await.unwrap();
            }
        }
    }
}
