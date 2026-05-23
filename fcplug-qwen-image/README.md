# Qwen Image Plugin

FlowCloudAI WASM 插件，用于适配阿里云百炼（DashScope）Qwen 图像生成 API。

---

## 支持模型

- `qwen-image-2.0-pro`
- `qwen-image-2.0`
- `qwen-image-max`
- `qwen-image-plus`

---

## 构建

```bash
cargo build --target wasm32-wasip2 --release
cargo fcplug build
```

---

## 接口

实现 WIT `mapper` 接口：

- `map-request`：将统一图像生成请求映射为 DashScope 多模态生成格式
- `map-response`：将 DashScope 响应映射回统一格式（提取图片 URL / Base64）
- `map-stream-line`：处理流式响应行（如适用）

---

## 许可证

MIT
