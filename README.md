# biliup-rs
[![Telegram](https://img.shields.io/badge/Telegram-Group-blue.svg?logo=telegram)](https://t.me/+IkpIABHqy6U0ZTQ5)

B站命令行投稿工具, 支持**短信登录**，**账号密码登录**，**扫码登录**，**浏览器登录**
登录后返回的cookie和token保存在cookie.json中，可用于其他项目。

本项目使用Rust, 可以作为lib被调用，理论上可以通过 [PyO3](https://github.com/PyO3/pyo3) 作为库提供给Python
和 [napi-rs](https://github.com/napi-rs/napi-rs) 给Node.js等进行调用……

[下载地址](https://github.com/ForgQi/biliup-rs/releases)
## USEAGE
投稿支持两种模式：
* 快速投稿，输入`biliup upload test1.mp4 test2.mp4`即可快速多p投稿
* 通过配置文件投稿，配置文件详见[config.yaml](examples/config.yaml)，
支持按照Unix shell style patterns来批量匹配视频文件，如
`/media/**/*.mp4` 匹配 media 及其子目录中的所有 mp4 文件
且可以自由调整视频标题、简介、标签等
```shell
$ biliup help upload
USAGE:
    biliup.exe upload [OPTIONS] [VIDEO_PATH]...

ARGS:
    <VIDEO_PATH>...    需要上传的视频路径,若指定配置文件投稿不需要此参数

OPTIONS:
    -c, --config <FILE>    指定配置文件
    -h, --help             Print help information
    -l, --line <LINE>      选择上传线路，支持kodo, bda2, qn, ws
```
查看完整用法命令行输入`biliup -h`
### Windows演示
登录：
```shell
.\biliup.exe login
```
![login](.github/resource/login.gif)

上传：
```shell
.\biliup.exe upload
```
![upload](.github/resource/upload.gif)

## SEE ALSO
* 自动录播投稿[工具](https://github.com/ForgQi/biliup)
* 基于此项目的[GUI版](https://github.com/ForgQi/Caution)

___
bilibili投稿模式分主要为fetch和直传两种，线路概览：
* bup（直传b站投稿系统，适合**国内**）
  * upos
    * bda2（百度）
    * qn（七牛）
    * ws（网宿）
* bupfetch （传至合作方后由b站投稿系统拉取，适合**国外**）
  * kodo （七牛）
  * bos（百度）
  * gcs（谷歌）
  * cos （腾讯）

b站在上传前会通过probe来返回几条线路，并发包测试从中选择响应时间较短的，
但对与国外的机器实际上不太准确，所以建议还是在实际测试后手动选择一条线路，
实际测试大部分国外机器在kodo线路3并发的情况下能达到60-90MiB/s的速度，理论上增加并发数能跑满带宽。
> 用户等级大于3，且粉丝数>1000，web端投稿不限制分p数量。b站web端将替换为[合集](https://www.bilibili.com/read/cv14762048)

对于不满足条件的账号，多p投稿只能依靠b站的投稿客户端，但是投稿客户端使用的线路与web端不同，
质量低于web端的线路，在国外机器会放大这一差距。所以本项目使用client的提交接口配合web端的上传线路，
弥补两者各自的不足。既可以多p上传，又提供了质量（速度和稳定性）较高的线路，且提供了web端不具备的手动切换线路功能。

