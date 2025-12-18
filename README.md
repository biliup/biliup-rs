# biliup-rs

[![Crates.io](https://img.shields.io/crates/v/biliup)](https://crates.io/crates/biliup)
![GitHub all releases](https://img.shields.io/github/downloads/forgqi/biliup-rs/total)
[![Telegram](https://img.shields.io/badge/Telegram-Group-blue.svg?logo=telegram)](https://t.me/+IkpIABHqy6U0ZTQ5)
[![Discord chat][discord-badge]][discord-url]

[discord-badge]: https://img.shields.io/discord/1015494098481852447.svg?logo=discord
[discord-url]: https://discord.gg/shZmdxDFB7
# ⚠️ 仓库已归档！请前往新仓库继续使用 ⚠️

> [!IMPORTANT]
> 
> 🚨 **重要通知：本仓库已停止维护，所有后续开发与更新已迁移至新仓库。**  
> 👉 请立即访问并使用新的项目地址：[biliup](https://github.com/biliup/biliup)

本仓库已归档，仅供参考历史记录。请勿在此提交新的 Issue 或 Pull Request。

----


## 🗃️ 以下内容为旧版 README，仅供参考。

<details>
B 站命令行投稿工具，支持**短信登录**、**账号密码登录**、**扫码登录**、**浏览器登录**以及**网页Cookie登录**，并将登录后返回的 cookie 和 token 保存在 `cookie.json` 中，可用于其他项目。

**文档地址**：<https://biliup.github.io/biliup-rs>

本项目使用 Rust，可以作为 lib 被调用，理论上可以通过 [PyO3](https://github.com/PyO3/pyo3) 作为库提供给 Python 和 [napi-rs](https://github.com/napi-rs/napi-rs) 给 Node.js 等进行调用。

[下载地址](https://github.com/ForgQi/biliup-rs/releases)

## Aspirations

### upload

- [x] bilibili
- [ ] 小红书（work-in-process）

### download

- [x] 斗鱼直播
- [x] 虎牙直播
- [x] B站直播
- [ ] 抖音live (coming soon)
- [ ] 快手live (coming soon)

## USAGE

投稿支持**直接投稿**和对现有稿件**追加投稿**：

- 快速投稿，输入 `biliup upload test1.mp4 test2.mp4` 即可快速多p投稿；
- 通过配置文件投稿，配置文件详见 [config.yaml](examples/config.yaml) ，支持按照 Unix shell style patterns 来批量匹配视频文件，如 `/media/**/*.mp4` 匹配 media 及其子目录中的所有 mp4 文件且可以自由调整视频标题、简介、标签等：

```shell
$ biliup help upload
上传视频

Usage: biliup upload [OPTIONS] [VIDEO_PATH]...

Arguments:
  [VIDEO_PATH]...  需要上传的视频路径,若指定配置文件投稿不需要此参数

Options:
      --submit <SUBMIT>              提交接口 [default: client] [possible values: client, app, web]
  -c, --config <FILE>                Sets a custom config file
  -l, --line <LINE>                  选择上传线路 [possible values: bda2, ws, qn, bldsa, tx, txa, bda, alia]
      --limit <LIMIT>                单视频文件最大并发数 [default: 3]
      --copyright <COPYRIGHT>        是否转载, 1-自制 2-转载 [default: 1]
      --source <SOURCE>              转载来源 [default: ]
      --tid <TID>                    投稿分区 [default: 171]
      --cover <COVER>                视频封面 [default: ]
      --title <TITLE>                视频标题 [default: ]
      --desc <DESC>                  视频简介 [default: ]
      --dynamic <DYNAMIC>            空间动态 [default: ]
      --tag <TAG>                    视频标签，逗号分隔多个tag [default: ]
      --dtime <DTIME>                延时发布时间，距离提交大于4小时，格式为10位时间戳
      --interactive <INTERACTIVE>    [default: 0]
      --mission-id <MISSION_ID>
      --dolby <DOLBY>                是否开启杜比音效, 0-关闭 1-开启 [default: 0]
      --hires <LOSSLESS_MUSIC>       是否开启 Hi-Res, 0-关闭 1-开启 [default: 0]
      --no-reprint <NO_REPRINT>      0-允许转载，1-禁止转载 [default: 0]
      --open-elec <OPEN_ELEC>        是否开启充电, 0-关闭 1-开启 [default: 0]
      --up-selection-reply           是否开启精选评论，仅提交接口为app时可用
      --up-close-reply               是否关闭评论，仅提交接口为app时可用
      --up-close-danmu               是否关闭弹幕，仅提交接口为app时可用
      --extra-fields <EXTRA_FIELDS>  自定义提交参数
  -h, --help                         Print help
```

- 下载视频：`./biliup download https://xxxx`
- 查看转码失败具体分p：`./biliup show BVxxxxx`
- 查看完整用法命令行输入 `biliup -h`

```shell
$ biliup help
Upload video to bilibili.

Usage: biliup.exe [OPTIONS] <COMMAND>

Commands:
  login     登录B站并保存登录信息
  renew     手动验证并刷新登录信息
  upload    上传视频
  append    是否要对某稿件追加视频
  show      打印视频详情
  dump-flv  输出flv元数据
  download  下载视频
  list      列出所有已上传的视频
  bind      绑定用户与代理(用户通过user_cookie参数指定)
  help      Print this message or the help of the given subcommand(s)

Options:
  -p, --proxy <PROXY>              配置代理
  -u, --user-cookie <USER_COOKIE>  登录信息文件 [default: cookies.json]
      --rust-log <RUST_LOG>        [default: tower_http=debug,info]
  -h, --help                       Print help
  -V, --version                    Print version
```

### 多账号支持

请在子命令**之前**通过 `-u` 或者 `--user-cookie` 参数传入 cookie 文件的路径（默认为当前目录下的 "cookies.json"）。例如：

```shell
$biliup -u user1.json login
$biliup --user-cookie user2.json upload ...
$biliup renew  # ./cookies.json
```

### 代理支持

请在子命令**之前**通过 `-p` 或者 `--proxy` 参数传入 代理 的地址。例如：
```powershell
biliup -p http://username:password@proxy.example.com:8080 upload
```
您也可以通过以下方式为用户绑定代理
```powershell
.\biliup.exe -p http://username:password@proxy.example.com:8080 -u myname.json login
```
通过这样登录的账号会**自动**生成myname-proxy.json文件记录您使用的代理，后续您可以直接使用其他功能
```powershell
.\biliup.exe -u myname.json upoload
```
如果代理文件发生变更您可以直接编辑myname-proxy.json文件
如果您的账号已经登录也可以使用以下命令自动创建代理配置文件
```powershell
.\biliup.exe -p http://username:password@proxy.example.com:8080 -u myname2.json bind
```
或者您可以手动建立yourname-proxy.json文件并填入"代理地址"
### Windows 演示

登录：

```powershell
biliup login
```

![login](.github/resource/login.gif)

上传：

```powershell
biliup upload
```

![upload](.github/resource/upload.gif)

## SEE ALSO

- 自动录播投稿[工具](https://github.com/ForgQi/biliup)
- 基于此项目的[GUI版](https://github.com/ForgQi/Caution)

___

bilibili 投稿模式分主要为 fetch 和直传两种，线路概览：

测速：<http://member.bilibili.com/preupload?r=ping>

- bup（直传b站投稿系统）
  - upos
    - [x] bda2（百度云）
    - [x] qn（七牛）
    - [x] alia（阿里云海外）
    - [x] bldsa (B站自建)
    - [x] tx (腾讯云EO)
    - [x] txa (腾讯云EO海外)
    - [x] bda (百度云海外)
- bupfetch （传至合作方后由b站投稿系统拉取，**已经长时间不可用**）
  - [x] ~~kodo（七牛）~~
  - [ ] ~~bos（百度）~~
  - [ ] ~~gcs（谷歌）~~
  - [x] ~~cos（腾讯）~~

 > 未选择上传线路时，在上传前会通过 probe 来返回几条线路，并发包测试从中选择响应时间较短的，正常情况下都会选择到良好的上传线路。
 > 如果自动选择的线路上传速度不佳，可以增大并发数或指定上述已支持选择的线路。
 > 理论上，增加并发数能加快上传速度，但部分线路存在并发数限制，请结合实际自行测试。

## TIPS

用户等级大于 3 ，且粉丝数 > 1000 ，Web 端投稿不限制分 P 数量。B 站 Web 端将替换为[合集](https://www.bilibili.com/read/cv14762048) 。

对于不满足条件的账号，多 P 投稿只能依靠 B 站的投稿客户端，但是投稿客户端使用的线路与 Web 端不同，质量低于 Web 端的线路，在国外机器会放大这一差距。所以本项目使用 client 的提交接口配合 Web 端的上传线路，弥补两者各自的不足。既可以多 P 上传，又提供了质量（速度和稳定性）较高的线路，且提供了 Web 端不具备的手动切换线路功能。

## For Developers

```shell
export DATABASE_URL="sqlite:data.db"
cargo sqlx db create
cargo sqlx migrate run --source .\crates\biliup-cli\migrations\
cargo sqlx prepare  --merged
cargo run -- server -b localhost
```
</details>