# Changelog

## [v0.1.1](https://github.com/zjp-CN/bilingual/tree/v0.1.1) (2021-11-25)

[Full Changelog](https://github.com/zjp-CN/bilingual/compare/v0.1.0...v0.1.1)

**Implemented enhancements:**

- 翻译账户信息 id 和 key 读取顺序 [\#27](https://github.com/zjp-CN/bilingual/issues/27)
- 写入译文时，处理 unwrap 错误 [\#25](https://github.com/zjp-CN/bilingual/issues/25)

**Fixed bugs:**

- 链接的 Text 为空时，影响输出结果 [\#26](https://github.com/zjp-CN/bilingual/issues/26)
- Md 调用 bytes\_paragraph 之后再调用 chars\_paragraph 会导致 self.bytes 重复 [\#21](https://github.com/zjp-CN/bilingual/issues/21)
- files 和 dirs 参数的 long name 反了 [\#19](https://github.com/zjp-CN/bilingual/issues/19)

**Closed issues:**

- 将 `config::API` 变成成携带数据的枚举体，每次选择一种翻译接口 [\#5](https://github.com/zjp-CN/bilingual/issues/5)

## [v0.1.0](https://github.com/zjp-CN/bilingual/tree/v0.1.0) (2021-11-20)

[Full Changelog](https://github.com/zjp-CN/bilingual/compare/430f68d496eba9f3740d153aeeb55e78f32b429d...v0.1.0)

**Implemented enhancements:**

- 输出到文件：增加 -M 和 -D 参数 [\#17](https://github.com/zjp-CN/bilingual/issues/17)
- 增加控制待翻译字符数策略 [\#15](https://github.com/zjp-CN/bilingual/issues/15)
- 所有 `User` 增加 limit 字段：每次请求时，API 所接受的字符上限。 [\#14](https://github.com/zjp-CN/bilingual/issues/14)
- 将 `dbg!` 的内容变成 log [\#11](https://github.com/zjp-CN/bilingual/issues/11)
- 只读取目录下的 md 文件 [\#8](https://github.com/zjp-CN/bilingual/issues/8)
- 输出 md 的 capacity 问题 [\#7](https://github.com/zjp-CN/bilingual/issues/7)

**Closed issues:**

- 重构 `cmd::Bilingual`：去除子命令，添加 --api 参数 [\#13](https://github.com/zjp-CN/bilingual/issues/13)
- 重构百度翻译 API `Response` 反序列化的部分 [\#12](https://github.com/zjp-CN/bilingual/issues/12)
- 命令行参数的翻译文本（`-q` 和位置参数）整合成一段文本进行请求 [\#4](https://github.com/zjp-CN/bilingual/issues/4)



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
