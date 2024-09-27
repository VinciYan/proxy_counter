#![deny(warnings)]

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::str::FromStr;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response, StatusCode, Uri};

use tokio::net::{TcpListener, TcpStream};

#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

// 图片下载计数器
static IMAGE_DOWNLOAD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8100));

    let listener = TcpListener::bind(addr).await?;
    println!("正在监听 http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(proxy))
                .with_upgrades()
                .await
            {
                println!("服务连接失败: {:?}", err);
            }
        });
    }
}

async fn proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    println!("收到请求: 方法={:?}, 路径={}, 头部={:?}", req.method(), req.uri().path(), req.headers());
    println!("请求: {:?}", req);

    if Method::CONNECT == req.method() {
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            eprintln!("服务器 IO 错误: {}", e);
                        };
                    }
                    Err(e) => eprintln!("升级错误: {}", e),
                }
            });

            Ok(Response::new(empty()))
        } else {
            eprintln!("CONNECT 主机不是 socket 地址: {:?}", req.uri());
            let mut resp = Response::new(full("CONNECT 必须连接到 socket 地址"));
            *resp.status_mut() = StatusCode::BAD_REQUEST;

            Ok(resp)
        }
    } else {
        // 检查是否是目标图片下载请求
        let is_target_image = req.uri().path().starts_with("/image-public/");

        if is_target_image {
            // 构建新的 URI
            let new_uri = format!("http://xxx.com:45004{}", req.uri().path());
            let new_uri = Uri::from_str(&new_uri).expect("无效的 URI");

            // 保存原始路径
            let original_path = req.uri().path().to_string();

            // 创建新的请求
            let (parts, body) = req.into_parts();
            let mut new_req = Request::new(body);
            *new_req.method_mut() = parts.method;
            *new_req.uri_mut() = new_uri;
            *new_req.version_mut() = parts.version;
            *new_req.headers_mut() = parts.headers;

            // 连接到实际的服务器
            let stream = TcpStream::connect(("xxx.com", 45004)).await.unwrap();
            let io = TokioIo::new(stream);

            let (mut sender, conn) = Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .handshake(io)
                .await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("连接失败: {:?}", err);
                }
            });

            let resp = sender.send_request(new_req).await?;

            // 如果是目标图片请求，增加计数器
            if resp.status().is_success() || resp.status() == StatusCode::NOT_MODIFIED {
                let count = IMAGE_DOWNLOAD_COUNT.fetch_add(1, Ordering::SeqCst);
                println!("目标图片请求成功。状态码: {}. 总计数: {}", resp.status(), count + 1);
            } else if resp.status() == StatusCode::NOT_FOUND {
                println!("目标图片不存在。路径: {}", original_path);
                let mut not_found_resp = Response::new(full("Image Not Found"));
                *not_found_resp.status_mut() = StatusCode::NOT_FOUND;
                return Ok(not_found_resp);
            }

            Ok(resp.map(|b| b.boxed()))
        } else {
            // 对于非目标图片请求，返回 404 Not Found
            let mut resp = Response::new(full("Not Found"));
            *resp.status_mut() = StatusCode::NOT_FOUND;
            Ok(resp)
        }
    }
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "客户端写入 {} 字节并接收 {} 字节",
        from_client, from_server
    );

    Ok(())
}