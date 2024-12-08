use core::net::lookup_host;
use noli::net::{SocketAddr, TcpStream};

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        // URLからホストを探す
        let ips = match lookup_host(&"example.com") {
            Ok(ips) => ips,
            Err(e) => return Err(Error::Network("Failed to find IP addresses: {:#?}", e)),
        };

        if ips.len() < 1 {
            return Err(Error::Network("Failed to find IP addresses".to_string()));
        }

        // intoメソッドでtupleからSocketAddrに変換
        let socket_addr: SocketAddr = (ips[0], port).into();

        // ホスト名、ポート番号をもとに接続（ストリーム）を作成
        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to connect to TCP stream".to_string(),
                ))
            }
        };

        // ホストに送るリクエストを構築
        // リクエストラインを追加
        let mut request = String::from("GET/");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // ヘッダを追加
        request.push_str("Host: ");
        request.push_str(&host);
        request.push("\n");
        request.push_str("Accept: text/html\n");
        request.push_str("Connection: close\n");
        request.push("\n");

        // リクエストの送信
        let _bytes_written = match stream.write(request.as_bytes()) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to send a request to TCP".to_string(),
                ))
            }
        };

        // レスポンスの受信
        let mut received = Vec::new();
        loop {
            let mut buf = [0u8; 4096];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Err(Error::Network(
                        "Failed to receive a request from TCP stream".to_string(),
                    ));
                }
            };
            if bytes_read == 0 {
                break;
            }
            received.extend_from_slice(&buf[..bytes_read]);
        }

        match core::str::from_utf8(&received) {
            Ok(response) => HttpResponse::new(response.to_string()),
            Err(e) => Err(Error::Network(format!("Invalid received response: {}", e))),
        }
    }
}
