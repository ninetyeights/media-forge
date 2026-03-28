# Media Forge

跨平台桌面媒体处理工具，基于 Tauri 2 + Svelte 5 + Rust 构建。专注于批量图片压缩、格式转换和自动化文件监听处理。

## 功能特性

### 图片处理
- 7 种压缩预设（无损、轻压缩、标准、极限、目标大小、Web 优化等）
- 格式互转：JPG、PNG、WebP、AVIF、GIF、BMP、TIFF
- 目标大小压缩（二分搜索最优质量）
- 等比缩放
- EXIF 信息剥离
- 文件大小过滤（设置最小/最大范围）
- 文件夹树形视图，可折叠展开目录层级
- 批量选择，支持按文件和按文件夹勾选
- 可配置并发处理数（1-16）
- 高性能编码：turbojpeg（SIMD 加速）、libwebp、oxipng + imagequant

### 文件监听
- 多文件夹独立监听，每个文件夹独立启停
- 每个文件夹独立配置：媒体类型、扩展名过滤、压缩预设、输出格式、输出目录、是否递归子目录
- 新文件自动处理，按文件夹配置的预设执行
- 文件写入完成检测（等待文件大小稳定后再处理）
- 防重复处理机制（计数器 + TTL 过期）
- 按文件夹分组的处理日志，带时间戳和处理详情
- 日志跨重启持久化
- Popover 弹出式设置面板

### 系统托盘
- 托盘图标 + 实时状态文字（监听中/已停止 + 数量）
- 每个文件夹的子菜单，显示状态和启停控制
- 从托盘直接开始/停止监听

### 全局设置
- 并发处理数
- 文件大小显示单位（1024 / 1000）
- 默认输出目录
- 系统通知开关
- 开机自启
- 所有设置自动持久化

### 界面
- macOS 毛玻璃 / Windows Mica 效果
- 暗色主题（Catppuccin 风格配色）
- 自定义标题栏
- 拖拽导入文件和文件夹
- 后台处理完成系统通知

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Tauri 2 |
| 前端 | Svelte 5 (Runes)、TypeScript、Tailwind CSS 4 |
| 后端 | Rust |
| 图片处理 | image、turbojpeg、webp、oxipng、imagequant |
| 文件监听 | notify 7、notify-debouncer-mini |
| 持久化 | tauri-plugin-store |

## 开发

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 生产构建
pnpm tauri build
```

## 构建发布

GitHub Actions 自动构建以下平台：
- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)

创建 tag 触发构建：
```bash
git tag v0.1.0
git push origin v0.1.0
```

## 未完成功能

### 已有 UI 但后端未实现
- **视频处理** — 编解码设置界面已完成（H.264、H.265、VP9、AV1），处理逻辑待实现
- **音频处理** — 设置界面已完成（码率、采样率、格式转换），处理逻辑待实现
- **水印** — 界面已完成，处理逻辑待实现
- **历史记录** — 页面占位，待实现

### 计划中
- 图片预览（处理前后对比）
- 批量重命名（支持模板）
- 单文件处理进度条

## 许可证

MIT
