# AGENTS.md

本文件包含在 rpipe 仓库中工作的智能编码代理的指南和命令。

## 项目概述

`rpipe` 是一个用 Rust 实现的命令行字符串处理工具，支持流式处理。二进制文件名为 `rp`，使用基于 token 的自定义解析系统进行数据处理流水线。

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
- 90+ 个测试分布在 17 个模块中（src/op/trim.rs, src/parse/token/op.rs, src/condition.rs 等）
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

### 模块结构
- 核心模块：`condition`, `config`, `err`, `input`, `op`, `output`, `parse`, `pipe`, `print`, `fmt`, `help`
- 解析模块分为 `args` 和 `token` 子目录
- 操作模块按功能划分：`replace`, `slice`, `trim`

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

### 文档风格
- 使用 `///` 进行公共 API 文档
- 整个代码库中的文档和注释使用中文
- 为枚举变体提供具体示例
- 使用 `CmdHelp` 派生宏生成命令行帮助

### 可见性规则
- 大多数项使用 `pub(crate)` 进行内部模块可见性
- 只有 `main.rs` 使用 crate 的公共项
- 解析子模块使用 `pub(in crate::parse)` 进行受限可见性

### 解析器架构
- 使用 `nom` 解析器组合器库
- 自定义错误类型 `RpParseErr` 结合 nom 错误和应用错误
- 常见解析结果的类型别名：`ConfigResult`, `OpResult` 等
- 分离参数（`args`）和 token（`token`）的解析

### 迭代器模式
- 核心抽象：`Pipe` 结构体包装 `Box<dyn Iterator<Item = String>>`
- 操作实现函数式模式：`op_map`, `op_filter`, `op_inspect`
- 强调流式处理和惰性求值

### 依赖使用
- `itertools`：增强迭代器操作
- `nom`：解析器组合器与 `nom-language` 用于错误报告
- `thiserror`：使用派生宏进行错误处理
- `ordered-float`：具有全序关系的浮点数值
- `unicase`：不区分大小写的字符串比较
- `regex`：正则表达式匹配
- `rand`：随机数生成
- `rt-format`：运行时字符串格式化
- `cmd-help`：用于命令行帮助的自定义过程宏（本地依赖）

### Windows 特定代码
- Windows 剪贴板支持的条件编译
- 在 Windows 平台使用 `clipboard-win` crate

### 构建配置
- 发布版本针对速度优化：`opt-level = 3`, `lto = true`
- 单一代码生成单元以获得更好的优化
- 发布版本中 `panic = "abort"` 以减少二进制大小
- 启用自动符号剥离

### TODO 注释
- 格式：`TODO YYYY-MM-DD HH:MM description`
- 包含用于跟踪的时间戳

## 开发说明

- 项目使用 Rust 2024 版本
- 二进制名称为 `rp`（在 Cargo.toml 中定义）
- 用于生成帮助文档的自定义过程宏 `cmd-help`
- 所有文档和注释使用中文
- 进行更改后，运行 `cargo clippy` 和 `cargo fmt` 以确保代码质量
