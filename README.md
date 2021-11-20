# bilingual

[<img alt="github" src="https://img.shields.io/github/license/zjp-CN/bilingual?color=blue" height="20">](https://github.com/zjp-CN/bilingual)
[<img alt="github" src="https://img.shields.io/github/issues/zjp-CN/bilingual?color=db2043" height="20">](https://github.com/zjp-CN/bilingual/issues)
[<img alt="build status" src="https://github.com/zjp-CN/bilingual/workflows/Release%20CI/badge.svg" height="20">](https://github.com/zjp-CN/bilingual/actions)

针对 markdown 文件的命令行翻译。
使用翻译云服务（百度、腾讯、小牛），
我整理了一份 [API 选择方案](https://github.com/zjp-CN/bilingual/issues/2)。

## 安装

1. 下载 [已编译的版本](https://github.com/zjp-CN/bilingual/releases)；
2. 或者源码编译

    ```console
    git clone https://github.com/zjp-CN/bilingual.git
    cargo build --release --features bin
    ```

## 使用

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




