# AGENTS.md

本文件包含在 rpipe 仓库中工作的智能编码代理的指南和命令。

## 项目概述

`rpipe` 是一个用 Rust 实现的命令行字符串处理工具，支持流式处理。二进制文件名为 `rp`，使用基于 token 的自定义解析系统进行数据处理流水线。
当前版本：0.3.1

## 项目结构

```
rpipe/
├── Cargo.toml              # 项目配置
├── Cargo.lock              # 依赖锁定
├── rustfmt.toml            # 格式化配置
├── build.rs                # 构建脚本（设置 BUILD_TIME）
├── README.md               # 项目文档
├── AGENTS.md               # 代理指南
├── .gitignore              # Git 忽略配置
├── .github/workflows/      # CI 配置
│   └── rust.yml
├── cmd_help/               # 本地过程宏 crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── data/                   # 测试数据
├── examples/               # 示例代码
└── src/                    # 源代码
    ├── main.rs             # 程序入口
    ├── lib.rs              # 库入口
    ├── condition.rs        # 条件表达式
    ├── config.rs           # 配置选项
    ├── err.rs              # 错误处理
    ├── fmt.rs              # 格式化
    ├── help.rs             # 帮助系统
    ├── input.rs            # 输入命令
    ├── output.rs           # 输出命令
    ├── pipe.rs             # 管道核心
    ├── print.rs            # 打印宏
    ├── op/                 # 操作命令
    │   ├── mod.rs          # 操作命令定义
    │   ├── replace.rs      # 替换操作
    │   ├── slice.rs        # 切片操作
    │   └── trim.rs         # 修剪操作
    └── parse/              # 解析器
        ├── mod.rs
        ├── args/           # 参数模式解析
        │   ├── mod.rs
        │   ├── condition.rs
        │   ├── config.rs
        │   ├── input.rs
        │   ├── op.rs
        │   └── output.rs
        └── token/          # Token 模式解析
            ├── mod.rs
            ├── condition.rs
            ├── config.rs
            ├── input.rs
            ├── op.rs
            └── output.rs
```

## 构建/开发命令

### 核心命令
```bash
# 构建项目
cargo build

# 构建发布版本（优化版）
cargo build --release

# 运行所有测试
cargo test

# 运行单个测试（按函数名）
cargo test test_trim_blank
cargo test test_parse_case
cargo test test_text_len_range

# 运行特定模块的测试
cargo test op::trim::tests
cargo test parse::token::op::tests
cargo test condition::tests

# 运行测试并显示输出
cargo test -- --nocapture

# 检查代码而不构建
cargo check

# 根据 rustfmt.toml 格式化代码
cargo fmt

# 运行 clippy 检查
cargo clippy

# 对所有目标和特性运行 clippy
cargo clippy --all-targets --all-features
```

### 测试策略
- 测试使用 `#[cfg(test)]` 和 `#[test]` 属性嵌入在源文件中
- 112 个测试分布在以下模块中：
  - `condition::tests` (11个)
  - `input::iter_tests` (10个)
  - `op::tests` (22个)
  - `op::trim::tests` (6个)
  - `op::slice::tests` (1个)
  - `op::replace::tests` (1个)
  - `fmt::tests` (1个)
  - `parse::args::condition::tests` (9个)
  - `parse::args::input::tests` (6个)
  - `parse::args::op::tests` (11个)
  - `parse::token::condition::tests` (9个)
  - `parse::token::input::tests` (6个)
  - `parse::token::op::tests` (11个)
  - `parse::token::config::tests` (2个)
  - `parse::token::output::tests` (2个)
  - `parse::token::tests` (4个)
- CI 流水线运行 `cargo build --verbose` 和 `cargo test --verbose`
- 示例测试名称：`test_trim_blank`, `test_parse_case`, `test_text_len_range`, `test_parse_sum`, `test_parse_replace`

## 代码风格指南

### 格式化配置
项目使用自定义 rustfmt.toml 设置：
- `max_width = 120` 字符
- `use_small_heuristics = "Max"`
- `imports_granularity = "Crate"` - 按 crate 分组导入
- `imports_layout = "Horizontal"` - 优先水平导入
- `fn_call_width = 120`, `chain_width = 120` - 方法链式调用
- `fn_args_layout = "Compressed"` - 紧凑函数参数
- `brace_style = "SameLineWhere"` - 左括号在同一行
- `empty_item_single_line = true` - 单行空项
- `format_strings = true` - 格式化字符串字面量
- `merge_derives = true` - 合并 derive 属性
- `struct_variant_width = 120` - 结构体变体宽度
- `enum_discrim_align_threshold = 0` - 枚举判别式对齐阈值

### 命名约定
- 类型：`PascalCase`（例如：`Config`, `RpErr`, `Pipe`）
- 函数/变量：`snake_case`（例如：`parse_configs`, `run`）
- 常量：`SCREAMING_SNAKE_CASE`（如果有）
- 枚举变体：`PascalCase`（例如：`Op::Peek`, `Condition::Yes`）
- 模块名：`snake_case`

### 导入组织
```rust
// 标准库导入优先
use std::iter::Peekable;
use std::str::FromStr;

// 外部 crate 其次
use itertools::Itertools;
use nom::error::{ContextError, ErrorKind};
use thiserror::Error;

// 内部模块最后（使用 crate::）
use crate::config::Config;
use crate::err::RpErr;
```

### 类型别名
`src/lib.rs` 中定义的常见类型别名：
- `type Integer = i64;`
- `type Float = f64;`
- 在其他模块中使用内部类型时使用 `crate::` 前缀

### 错误处理
- 使用 `thiserror` 处理自定义错误类型
- 主错误类型：`RpErr` 包含不同失败模式的错误代码
- 为帮助文档实现 `CmdHelp` 派生宏
- 使用 `Result<T, RpErr>` 进行可能失败的操作
- 错误消息格式：`[ErrorCode:N] description: {details}`
- 错误代码按顺序编号（1-14），必须按顺序维护

| 错误码 | 名称 | 描述 |
|--------|------|------|
| 1 | ParseTokenErr | 解析配置 Token 失败 |
| 2 | ArgParseErr | 参数解析失败 |
| 3 | MissingArg | 命令缺少有效参数 |
| 4 | UnexpectedRemaining | 参数内容无法完全解析 |
| 5 | UnknownArgs | 未知参数 |
| 6 | ReadClipboardTextErr | 从剪贴板读取失败 |
| 7 | ReadFromFileErr | 从文件读取失败 |
| 8 | WriteToClipboardErr | 写入剪贴板失败 |
| 9 | OpenFileErr | 打开文件失败 |
| 10 | WriteToFileErr | 写入文件失败 |
| 11 | FormatStringErr | 格式化字符串失败 |
| 12 | ParseRegexErr | 解析正则表达式失败 |
| 13 | ParseNumErr | 解析数值失败 |
| 14 | InvalidNonNegativeIntArg | 无效的非负整数参数 |

### 文档风格
- 使用 `///` 进行公共 API 文档
- 整个代码库中的文档和注释使用中文
- 为枚举变体提供具体示例
- 使用 `CmdHelp` 派生宏生成命令行帮助

### 可见性规则
- 大多数项使用 `pub(crate)` 进行内部模块可见性
- 只有 `main.rs` 使用 crate 的公共项
- 解析子模块使用 `pub(in crate::parse)` 进行受限可见性

### TODO 注释
- 格式：`TODO YYYY-MM-DD HH:MM description`
- 包含用于跟踪的时间戳

## 命令参考

### 配置选项

- `-V, --version` - 打印版本信息
- `-h, --help` - 打印帮助信息（支持主题：opt, in, op, out, fmt, cond, code）
- `-v, --verbose` - 执行前打印流水线详情
- `-d, --dry-run` - 仅解析，不执行
- `-n, --nocase` - 全局忽略大小写
- `-s, --skip-err` - 全局忽略错误
- `-t, --token` - 以 Token 模式解析下一个参数

### 输入命令

支持的输入方式：

- `:in` - 从标准输入读取（默认输入）
- `:file <file>[ <file>...]` - 从文件读取
- `:clip` - 从剪贴板读取（仅 Windows）
- `:of <text>[ <text>...]` - 使用直接字面值
- `:gen <start>[,[<end>][,<step>]][ <fmt>]` - 生成整数序列
- `:repeat <value>[ <count>]` - 重复字面值

### 操作命令

#### 访问操作
- `:peek[ <file>][ append][ lf|crlf]` - 打印每个值到标准输出或文件

#### 转换操作
- `:upper` - 转为 ASCII 大写
- `:lower` - 转为 ASCII 小写
- `:case` - 切换 ASCII 大小写

#### 替换操作
- `:replace <from> <to>[ <count>][ nocase]` - 替换字符串

#### 修剪操作
- `:trim[ <pattern>[ nocase]]` - 去除首尾子串
- `:ltrim[ <pattern>[ nocase]]` - 去除首部子串
- `:rtrim[ <pattern>[ nocase]]` - 去除尾部子串
- `:trimc[ <pattern>[ nocase]]` - 去除首尾字符
- `:ltrimc[ <pattern>[ nocase]]` - 去除首部字符
- `:rtrimc[ <pattern>[ nocase]]` - 去除尾部字符
- `:trimr <regex>` - 去除首尾匹配正则的字串
- `:ltrimr <regex>` - 去除首部匹配正则的字串
- `:rtrimr <regex>` - 去除尾部匹配正则的字串
- `:reg <regex>[ <count>]` - 正则匹配并替换为匹配内容

#### 减少操作
- `:limit <count>` - 保留前 N 个数据
- `:skip <count>` - 丢弃前 N 个数据
- `:slice [ <range>...]` - 切片（范围格式：`<start>,<end>`）
- `:uniq[ nocase]` - 去重
- `:sum[ <fmt>]` - 累加数值（支持格式化）
- `:join[ <delimiter>[ <prefix>[ <postfix>[ <batch>]]]]` - 合并数据
- `:drop <condition>` - 根据条件丢弃数据
- `:take <condition>` - 根据条件保留数据
- `:drop while <condition>` - 持续丢弃直到条件不满足
- `:take while <condition>` - 持续保留直到条件不满足
- `:count` - 统计数据数量

#### 排序操作
- `:sort[ num [<default>]][ nocase][ desc][ random]` - 排序
  - `num` - 按数值排序（可指定默认值）
  - `nocase` - 忽略大小写（仅字典序）
  - `desc` - 逆序
  - `random` - 随机排序

### 输出命令

支持的输出方式：

- `:to out` - 输出到标准输出（默认输出）
- `:to file <file>[ append][ lf|crlf]` - 输出到文件
- `:to clip[ lf|crlf]` - 输出到剪贴板（仅 Windows）

### 条件表达式

#### 字符串长度
- `[not] len [<min>],[<max>]` - 按长度范围选择
- `[not] len <len>` - 按特定长度选择

#### 数值
- `[not] num [<min>],[<max>]` - 按数值范围选择
- `[not] num <spec>` - 按特定数值选择
- `[not] num[ [integer|float]]` - 按整数/浮点数选择

#### 文本属性
- `[not] upper` - 全部大写
- `[not] lower` - 全部小写
- `[not] ascii` - 全部 ASCII
- `[not] nonascii` - 全部非 ASCII
- `[not] empty` - 空字符串
- `[not] blank` - 全部空白字符

#### 正则匹配
- `[not] reg <exp>` - 匹配正则表达式

## 核心设计

### 模块结构
- 核心模块：`condition`, `config`, `err`, `input`, `op`, `output`, `parse`, `pipe`, `print`, `fmt`, `help`
- 解析模块分为 `args` 和 `token` 子目录
- 操作模块按功能划分：`replace`, `slice`, `trim`

### 解析器架构
- 使用 `nom` 解析器组合器库
- 自定义错误类型 `RpParseErr` 结合 nom 错误和应用错误
- 常见解析结果的类型别名：`ConfigResult`, `OpResult` 等
- 分离参数（`args`）和 token（`token`）的解析

**双解析模式：**

1. **Args 模式** (`parse::args`) - 命令行参数解析
   - 按空格分割参数
   - 支持引号包裹（单引号/双引号）
   - 支持转义字符：`\\`, `\0`, `\ `, `\"`, `\'`, `\r`, `\n`, `\t`
   - 命令以 `:` 开头

2. **Token 模式** (`parse::token`) - 使用 nom 解析器组合器
   - 类 POSIX Shell 参数解析规则
   - 支持单引号（不转义）、双引号（支持转义）
   - 支持 `\:` 转义避免被识别为命令

### 迭代器模式
- 核心抽象：`Pipe` 结构体包装 `Box<dyn Iterator<Item = String>>`
- 操作实现函数式模式：`op_map`, `op_filter`, `op_inspect`
- 强调流式处理和惰性求值

### 关键设计模式
- **Num 类型**：统一的数值类型，支持整数和浮点数自动转换，实现 `std::iter::Sum` 用于流式累加
- **Pipe 结构体**：包装 `Box<dyn Iterator<Item = String>>`，支持惰性求值和流式处理
- **条件选择系统**：`Condition` 和 `Select` 枚举支持复杂的条件表达式，包括文本、数值、正则匹配等
- **双解析模式**：支持 args（参数模式）和 token（令牌模式）两种解析方式
- **错误处理**：使用 `thiserror` 生成错误代码，实现 `Termination` trait 提供友好的退出信息

## 依赖使用

| 依赖 | 版本 | 用途 |
|------|------|------|
| itertools | 0.14.0 | 迭代器增强 |
| nom | 8.0.0 | 解析器组合器 |
| nom-language | 0.1.0 | 解析器错误信息辅助 |
| thiserror | 2.0.17 | 错误处理 |
| ordered-float | 5.1.0 | 浮点数排序 |
| unicase | 2.9.0 | 不区分大小写比较 |
| rand | 0.9.2 | 随机数生成 |
| rt-format | 0.3.1 | 运行时字符串格式化 |
| regex | 1.12.2 | 正则表达式匹配 |
| rustc-hash | 2.1.1 | 高性能哈希算法 |
| cmd-help | 本地 0.1.0 | 命令行帮助过程宏 |
| clipboard-win | 5.4.1 | Windows 剪贴板支持（条件编译）|
| time | 0.3.45 | 处理时间（构建依赖）|
| syn | 2.0 | 过程宏 AST 解析 |
| quote | 1.0 | 过程宏代码生成 |

## 性能优化

- 使用 `rustc_hash::FxHashSet` 替代标准库 `HashSet` 提高哈希性能
- `optimized_char_count` 函数对 ASCII 文本使用 O(1) 的字节长度计算
- 大小写转换前进行 ASCII 检查避免不必要的操作
- 使用 `Cow<str>` 减少字符串分配（在替换操作中）

## Windows 特定代码

- Windows 剪贴板支持的条件编译 `#[cfg(windows)]`
- 在 Windows 平台使用 `clipboard-win` crate
- 支持从剪贴板读取（`:clip` 输入）和写入剪贴板（`:to clip` 输出）
- 可配置换行符格式（LF 或 CRLF）

## 构建配置

发布版本针对性能优化：
- `opt-level = 3` - 速度优化
- `lto = true` - 启用链接时优化
- `codegen-units = 1` - 单一代码生成单元以获得更好的优化
- `panic = "abort"` - 减少二进制大小
- `strip = true` - 启用自动符号剥离

## 开发说明

- 项目使用 Rust 2024 版本
- 二进制名称为 `rp`（在 Cargo.toml 中定义）
- 用于生成帮助文档的自定义过程宏 `cmd-help`
- 所有文档和注释使用中文
- 进行更改后，运行 `cargo clippy` 和 `cargo fmt` 以确保代码质量

## 使用示例

```bash
# Token 模式
rp -t ':in :uniq :to out'

# Args 模式 - 从文件读取、修剪空格、输出到文件
rp :file input.txt :trim :to file output.txt

# 生成序列
rp :gen 0,10 "n{v}" :to out

# 条件过滤 - 只保留数值 1-100 的数据
rp :in :take num 1,100 :to out

# 排序并限制前 10 个
rp :file data.txt :sort num desc :limit 10 :to out

# 替换操作
rp :in :replace foo bar :to out

# 去重并排序
rp :file input.txt :uniq :sort :to out

# 累加数值
rp :in :sum :to out

# 正则匹配
rp :in :reg "\d+" :to out
```
