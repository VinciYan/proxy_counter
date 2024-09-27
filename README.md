# 图片代理和统计服务

这是一个使用 Rust 编写的高性能图片代理服务器。它专门用于代理和统计图片资源的访问

例如：

http://127.0.0.1:8100/image-public/0a1e65f4-7ced-4ef0-ba7d-12ec4d14a0d4.png
->http://xxx.com:45004/image-public/0a1e65f4-7ced-4ef0-ba7d-12ec4d14a0d4.png

## 项目特点

- 高性能：使用 Rust 语言编写
- 异步处理：基于 Tokio 运行时，实现高并发的异步 I/O 操作
- 精确统计：准确记录目标图片的访问次数

## 技术栈

- Rust 编程语言
- Hyper：用于 HTTP 服务器和客户端的快速、安全框架
- Tokio：异步运行时，提供高效的 I/O 操作

## 功能介绍

1. HTTP 代理：
    - 监听本地端口（默认 8100），接收 HTTP 请求
    - 访问目标图片（路径以 /image-public/ 开头）将被代理，转发到配置的目标服务器

2. 图片访问统计：
    - 精确统计目标图片的访问次数

3. 请求日志：
    - 详细记录每个请求的方法、路径和头部信息
    - 输出响应状态码和图片访问计数

4. 错误处理：
    - 对于错误图片的请求，返回 404 Not Found 响应

## 使用方法

1. 克隆仓库：
   ```
   git clone [仓库URL]
   ```

2. 编译项目：
   ```
   cargo build --release
   ```

3. 运行服务器：
   ```
   cargo run --release
   ```

4. 服务器将在 `http://localhost:8100` 上启动

5. 配置您的客户端或浏览器使用此地址作为 HTTP 代理

6. 访问目标图片（路径以 /image-public/ 开头）将被代理并计数

7. 运行效果

```
收到请求: 方法=GET, 路径=/image-public/0a1e65f4-7ced-4ef0-ba7d-12ec4d14a0d4.png, 头部={"content-type": "application/json", "user-agent": "PostmanRuntime/7.42.0", "accept": "*/*", "postman-token": "9fe5ee1a-ad8e-4e0d-8f65-e82090115795", "host": "127.0.0.1:8100", "accept-encoding": "gzip, deflate, br", "connection": "keep-alive", "content-length": "75"}       
请求: Request { method: GET, uri: /image-public/0a1e65f4-7ced-4ef0-ba7d-12ec4d14a0d4.png, version: HTTP/1.1, headers: {"content-type": "application/json", "user-agent": "PostmanRun
time/7.42.0", "accept": "*/*", "postman-token": "9fe5ee1a-ad8e-4e0d-8f65-e82090115795", "host": "127.0.0.1:8100", "accept-encoding": "gzip, deflate, br", "connection": "keep-alive", "content-length": "75"}, body: Body(Streaming) }
响应状态码: 200 OK
目标图片请求成功。状态码: 200 OK. 总计数: 3
```