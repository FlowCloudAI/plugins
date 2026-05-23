# DeepSeek LLM Plugin

FlowCloudAI WASM 插件，用于适配 DeepSeek API。

---

## 支持模型

- `deepseek-v4-flash`
- `deepseek-v4-pro`

---

## 构建

```bash
cargo build --target wasm32-wasip2 --release
cargo fcplug build
```

---

## 接口

实现 WIT `mapper` 接口：

- `map-request`：将统一请求映射为 DeepSeek `/chat/completions` 格式
- `map-response`：将 DeepSeek 响应映射回统一格式
- `map-stream-line`：处理 SSE 流式行

---

## 许可证

MIT
