# Qwen LLM Plugin

FlowCloudAI WASM 插件，用于适配阿里云百炼（DashScope）Qwen LLM API。

---

## 支持模型

- `qwen3.7-max`
- `qwen3.7-plus`
- `qwen3.6-flash`

---

## 构建

```bash
cargo build --target wasm32-wasip2 --release
cargo fcplug build
```

---

## 接口

实现 WIT `mapper` 接口：

- `map-request`：将统一请求映射为 DashScope `/compatible-mode/v1/chat/completions` 格式
- `map-response`：将 DashScope 响应映射回统一格式
- `map-stream-line`：处理 SSE 流式行

---

## 许可证

MIT
