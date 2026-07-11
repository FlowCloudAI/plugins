# 智谱 Image 插件

FlowCloudAI WASM 插件，用于适配智谱 GLM-Image 与 CogView 图片生成 API。

智谱同步接口目前每次返回一张图片，且不接收参考图，因此插件只声明文生图能力。

## 支持模型

- `glm-image`
- `cogview-4-250304`
- `cogview-4`
- `cogview-3-flash`

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
