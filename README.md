# bilingual

[<img alt="github" src="https://img.shields.io/github/license/zjp-CN/bilingual?color=blue" height="20">](https://github.com/zjp-CN/bilingual)
[<img alt="github" src="https://img.shields.io/github/issues/zjp-CN/bilingual?color=db2043" height="20">](https://github.com/zjp-CN/bilingual/issues)
[<img alt="build status" src="https://github.com/zjp-CN/bilingual/workflows/Release%20CI/badge.svg" height="20">](https://github.com/zjp-CN/bilingual/actions)
[<img alt="crates.io" src="https://img.shields.io/crates/v/bilingual?style=flat&color=fc8d62&logo=rust&label=bilingual" height="20">](https://crates.io/crates/bilingual)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-translation_api_cn-66c2a5?style=flat&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/translation-api-cn)

针对 markdown 文件的命令行翻译。
使用翻译云服务（百度、腾讯、小牛），
此外：
- 我整理了一份 [API 选择方案](https://github.com/zjp-CN/bilingual/issues/2)；
- 拆分出两个辅助库：它们有最少依赖，尽可能地降低与命令行功能的耦合，增加通用性。
    - [translation-api-cn](https://docs.rs/translation-api-cn)：定义翻译的网络（请求和接收）接口结构体。
    - [bilingual](https://docs.rs/bilingual/0.1.0/bilingual/)：根据字节/字符上限抽取 (extract) md 文本。

该 cmdline tool 的目的：翻译 md 文件。和网页翻译一样，md 文件也包含很多样式（tag）。

不足：
- 非异步 I/O：由于 API 调用的 QPS 限制，异步请求暂时不是考虑的重点。
- 可以处理简单的 md 样式，但是复杂样式尚未支持：含有表格、HTML 样式的地方，其翻译内容不会是你想要的结果。
  （见 [#11](https://github.com/zjp-CN/bilingual/issues/10)）
- 未支持更多翻译 API 服务：性价比是我考虑翻译 API 的主要原因，见 [#2](https://github.com/zjp-CN/bilingual/issues/2)。

## 安装

1. 下载 [已编译的版本](https://github.com/zjp-CN/bilingual/releases)；
2. 或者 cargo 安装：

    ```console
    CARGO_PROFILE_RELEASE_LTO=yes CARGO_PROFILE_RELEASE_OPT_LEVEL=3 cargo install bilingual --features bin
    ```

2. 或者源码编译：

    ```console
    git clone https://github.com/zjp-CN/bilingual.git
    cd bilingual
    cargo build --release --features bin
    ```

## 使用

`bilingual.toml` 样例：

```toml
[baidu]
appid = "xxxxxxxxxxxxxxxxx"
key = "xxxxxxxxxxxxxxxxxxxx"
# limit = { bytes = 6000 }

[tencent]
id = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
key = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# limit = { chars = 2000 }

[niutrans]
key = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# limit = { chars = 5000 }
```

命令行帮助：

```md
$ bilingual --help

Usage: bilingual [<multiquery...>] -a <api> [-i <id>] [-k <key>] [-f <from>] [-t <to>] [-q <singlequery>] [-m <input
-dirs...>] [-d <input-files...>] [-M <output-dirs...>] [-D <output-files...>] [-r] [--forbid-dir-creation] [--toml <
toml>]

【bilingual】 作者：苦瓜小仔

针对 markdown 文件的命令行翻译。使用 `bilingual --help` 查看此帮助说明。

例子：
* `bilingual -a baidu multi queries -q single-query`
* `bilingual -a tencent -m xx.md`
* `bilingual -a niutrans -d ./dir-path`
* `bilingual -a tencent \#\ 标题 正文：模拟\ markdown\ 文件的内容。 -f zh -t en`
* `bilingual -a tencent -m xx.md -M xx-中文.md -d path -D path-中文`

注意：本程序使用翻译云服务，因此需要自行申请翻译 API。
      命令行提供的 id 和 key 会覆盖掉配置文件的信息。
      换言之，未提供命令行的 appid 和 key，则使用配置文件的信息。
      建议将账户信息统一写在当前路径下的 bilingual.toml 目录（或者由 --toml 指定的路径）。

Options:
  -a, --api         翻译 API。必选参数。目前支持：baidu | tencent | niutrans。
  -i, --id          翻译 API 账户的 id。
  -k, --key         翻译 API 账户的 key。
  -f, --from        原语言。默认为 en。
  -t, --to          目标语言。默认为 zh。
  -q, --singlequery 单行翻译文本：翻译文本内特殊符号以 `\` 转义。翻译的顺序位于所有多行翻译文本之后。
  -m, --input-dirs  md 文件的输入路径。此工具把读取到的文件内容只当作 md 文件进行处理。且不修改 API 返回的任何内容。
  -d, --input-files 输入目录。此工具只识别和读取目录下以 `.md` 结尾的文件。
  -M, --output-dirs md 文件的输出路径。默认在输入的文件路径下，但是翻译后的文件名会增加 `--to` 标识。
  -D, --output-files
                    输出目录。默认在输入的目录旁，但是翻译后的目录会增加 `--to` 标识。
  -r, --replace-file
                    如果输出文件已存在，是否替换。默认不替换。
  --forbid-dir-creation
                    在输出文件夹时不存在时，禁止创建输出文件夹。默认总是创建新文件夹。
  --toml            配置文件 bilingual.toml 的路径。默认是当前目录下，即 `./bilingual.toml`。
  --help            display usage information
```

实际使用例子：
- `bilingual -a tencent "# 标题" "正文：模拟 markdown 文件的内容。" -f zh -t en`（等价于使用 `\` 转义的例子）结果：
 
    ```md
    # 标题

    # Title

    正文：模拟 markdown 文件的内容。

    Body: simulates the contents of the markdown file.
    ```

- [8_6_io_eventqueue-zh.md](https://github.com/zjp-CN/bilingual/blob/main/assets/8_6_io_eventqueue-zh.md)：源文件来自于[此处](https://github.com/cfsamson/book-exploring-async-basics)
- [markdown-it](https://github.com/zjp-CN/bilingual/blob/main/assets/markdown-it.md) 的 [各种 API 翻译后的版本](https://github.com/zjp-CN/bilingual/blob/main/assets/markdown-it)：源文件来自于[此处](https://markdown-it.github.io/)


