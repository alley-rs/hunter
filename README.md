# hunter

轻量简单的 trojan-go 图形化桌面客户端。

仅限 Windows 和 macOS 平台，Linux 桌面生态无统一命令，无法完成对所有 Linux 发行版的统一适配。

## 介绍

本软件通过 trojan-go 二进制文件与其配置文件并设置系统代理为 pac 实现代理。

前端使用 solidjs 和我自己写的组件完成，核心逻辑在后端的 rust 完成。因为后端有很多代码直接复用的之前的内部软件代码，为了发布公开版又添加了一些代码，所以整体代码逻辑不太好。

## 安装

在 release latest 中下载对应平台和架构的安装包即可。

## 使用

需要填写的参数不多，依次填写后启用节点即开启代理。

![app](./docs/images/app.avif)

缺少 trojan-go 时会自动下载对应系统的最新版：

![app](./docs/images/downloading.avif)
