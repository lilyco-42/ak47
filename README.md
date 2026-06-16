# 🔫 AK47 — 词频分析器

> 一个基于 Rust 的桌面词语分析工具，提供 Windows GUI 和 Web 可视化界面。

## ✨ 功能

- 📂 打开 `.txt`、`.md`、`.lrc` 等文本文件
- 🔪 基于 [jieba-rs](https://github.com/messense/jieba-rs) 的中文分词
- 📊 统计词频，按出现次数排序展示
- ☁️ 生成词云 + 柱状图 HTML 可视化（基于 ECharts）
- 🖥️ 原生 Windows GUI（winsafe）

## 📸 界面

程序启动后：

1. 点击 **📁 打开文件** 选择文本文件
2. 自动分词统计，结果显示在列表中
3. 点击 **📊 生成词云** 在浏览器中打开可视化页面
4. **🗑 清除** 重置当前状态

## 🚀 下载

前往 [Releases](https://github.com/lilyco-42/ak47/releases) 页面下载最新 `ak47-vX.Y.Z-x86_64-pc-windows-msvc.zip`，解压后双击 `ak47.exe` 即可运行。

## 🛠️ 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust 2024 edition |
| GUI | [winsafe](https://github.com/rodrigocfd/winsafe) |
| 分词 | [jieba-rs](https://github.com/messense/jieba-rs) |
| 文件选择 | [rfd](https://github.com/PolyMeilex/rfd) |
| 序列化 | [serde_json](https://github.com/serde-rs/json) |
| 可视化 | [ECharts](https://echarts.apache.org/) + echarts-wordcloud |

## 🔧 本地构建

```bash
# 需要 Rust 工具链 (MSVC)
git clone https://github.com/lilyco-42/ak47.git
cd ak47
cargo build --release
```

## 📁 项目结构

```
ak47/
├── src/
│   └── main.rs          # 全部源码（GUI + 词频统计 + HTML 生成）
├── .github/workflows/
│   ├── ci.yml           # CI：fmt → clippy → build
│   └── release.yml      # 推送 v* 标签自动构建发布
└── Cargo.toml
```
