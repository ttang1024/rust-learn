# rust

[English](README.md) | **简体中文**

学习资料：  
Rust 程序设计语言：https://doc.rust-lang.org/book/  
中文版：https://kaisery.github.io/trpl-zh-cn/  

标准库文档：https://doc.rust-lang.org/std/prelude/index.html  

- [rust](#rust)
  - [0. 仓库结构](#0-仓库结构)
  - [1. Rust 的安装(Mac)](#1-rust-的安装mac)
  - [2. Cargo](#2-cargo)
    - [2.1. 使用 Cargo 创建项目](#21-使用-cargo-创建项目)
      - [2.1.1. 从 crates.io 引入依赖包](#211-从-cratesio-引入依赖包)
      - [2.1.2. 从其它注册服务引入依赖包](#212-从其它注册服务引入依赖包)
      - [2.1.3. 引入 git 仓库作为依赖包](#213-引入-git-仓库作为依赖包)
      - [2.1.4. 通过路径引入本地依赖包](#214-通过路径引入本地依赖包)
      - [2.1.5. 根据平台引入依赖](#215-根据平台引入依赖)
      - [2.1.6. \[dev-dependencies\]](#216-dev-dependencies)
      - [2.1.7. \[build-dependencies\]](#217-build-dependencies)
      - [2.1.8 选择 features](#218-选择-features)
      - [2.1.9. 在 Cargo.toml 中重命名依赖](#219-在-cargotoml-中重命名依赖)
    - [2.2. 构建并运行 Cargo 项目](#22-构建并运行-cargo-项目)
    - [2.3. Cargo.toml 格式](#23-cargotoml-格式)
    - [2.4. 发布（release）构建](#24-发布release构建)
  - [3. 基础语法](#3-基础语法)
    - [3.1. 变量](#31-变量)
    - [3.2. 常量](#32-常量)
    - [3.3. 隐藏](#33-隐藏)
  - [4. 数据类型](#4-数据类型)
    - [4.1. 标量类型](#41-标量类型)
      - [4.1.1. 整型](#411-整型)
      - [4.1.2. 浮点型](#412-浮点型)
    - [4.2. 数值运算](#42-数值运算)
    - [4.3. 布尔型](#43-布尔型)
    - [4.4. 字符类型](#44-字符类型)
    - [4.5. 复合类型](#45-复合类型)
      - [4.5.1. 元组类型](#451-元组类型)
      - [4.5.2. 数组类型](#452-数组类型)
  - [5. 函数](#5-函数)
    - [5.1. 参数](#51-参数)
    - [5.2. 语句和表达式](#52-语句和表达式)
    - [5.3. 函数的返回值](#53-函数的返回值)
  - [6. 控制流](#6-控制流)
    - [6.1. if 表达式](#61-if-表达式)
    - [6.2. 使用循环重复执行](#62-使用循环重复执行)
      - [6.2.1. 使用 loop 重复执行代码](#621-使用-loop-重复执行代码)
      - [6.2.2. while 条件循环](#622-while-条件循环)
      - [6.2.3. 使用 for 遍历集合](#623-使用-for-遍历集合)
  - [7. 所有权](#7-所有权)
    - [7.1. 引用与借用](#71-引用与借用)
    - [7.2. Slice 类型](#72-slice-类型)
  - [8. 结构体](#8-结构体)
    - [8.1. 方法](#81-方法)
  - [9. 枚举](#9-枚举)
  - [10. match 控制流](#10-match-控制流)
  - [11. if let 控制流](#11-if-let-控制流)
  - [12. 泛型和特征](#12-泛型和特征)
    - [12.1. 结构体中使用泛型](#121-结构体中使用泛型)
    - [12.2. 枚举中使用泛型](#122-枚举中使用泛型)
    - [12.3. 方法中使用泛型](#123-方法中使用泛型)
    - [12.4. const 泛型（Rust 1.51 版本引入的重要特性）](#124-const-泛型rust-151-版本引入的重要特性)
      - [12.4.1 const 泛型表达式](#1241-const-泛型表达式)
    - [12.5. 泛型的性能](#125-泛型的性能)
    - [12.6. 特征 Trait](#126-特征-trait)
      - [12.6.1. 为类型实现特征](#1261-为类型实现特征)
      - [12.6.2. 特征定义与实现的位置(孤儿规则)](#1262-特征定义与实现的位置孤儿规则)
      - [12.6.3. 默认实现](#1263-默认实现)
      - [12.6.4. 使用特征作为函数参数](#1264-使用特征作为函数参数)
      - [12.6.5. 特征约束(trait bound)](#1265-特征约束trait-bound)
      - [12.6.6. 多重约束](#1266-多重约束)
      - [12.6.7. Where 约束](#1267-where-约束)
      - [12.6.8. 使用特征约束有条件地实现方法或特征](#1268-使用特征约束有条件地实现方法或特征)
      - [12.6.9. 函数返回中的 impl Trait](#1269-函数返回中的-impl-trait)
      - [12.6.10. 为自定义类型实现 + 操作](#12610-为自定义类型实现--操作)
    - [12.7. 特征对象](#127-特征对象)
  - [13. 动态数组 Vector](#13-动态数组-vector)
    - [13.1. 创建动态数组](#131-创建动态数组)
    - [13.2. 更新 Vector](#132-更新-vector)
    - [13.3. 从 Vector 中读取元素](#133-从-vector-中读取元素)
    - [13.4. 同时借用多个数组元素](#134-同时借用多个数组元素)
    - [13.5. 迭代遍历 Vector 中的元素](#135-迭代遍历-vector-中的元素)
    - [13.6. 存储不同类型的元素](#136-存储不同类型的元素)
    - [13.7. Vector 常用方法](#137-vector-常用方法)
    - [13.8. Vector 的排序](#138-vector-的排序)
  - [14. HashMap](#14-hashmap)
    - [14.1. 创建 HashMap](#141-创建-hashmap)
    - [14.2. 所有权转移](#142-所有权转移)
    - [14.3. 查询 HashMap](#143-查询-hashmap)
    - [14.4. 更新 HashMap 中的值](#144-更新-hashmap-中的值)
    - [14.5. 哈希函数](#145-哈希函数)
  - [15. 生命周期](#15-生命周期)
    - [15.1. 悬垂指针和生命周期](#151-悬垂指针和生命周期)
    - [15.2. 借用检查](#152-借用检查)
    - [15.3. 函数中的生命周期](#153-函数中的生命周期)
  - [16. 方法 Method](#16-方法-method)
    - [16.1. 定义方法](#161-定义方法)
    - [16.2. self、\&self 和 \&mut self](#162-selfself-和-mut-self)
    - [16.3. 方法名跟结构体字段名相同](#163-方法名跟结构体字段名相同)
    - [16.4. 带有多个参数的方法](#164-带有多个参数的方法)
    - [16.5. 关联函数](#165-关联函数)
    - [16.6. 多个 impl 定义](#166-多个-impl-定义)
    - [16.7. 为枚举实现方法](#167-为枚举实现方法)
  - [17. 使用 macro\_rules! 来创建宏](#17-使用-macro_rules-来创建宏)
    - [17.1. 指示符](#171-指示符)
    - [17.2. 重载](#172-重载)
    - [17.3. 重复](#173-重复)
    - [17.4. DRY (不写重复代码)](#174-dry-不写重复代码)
    - [17.5. DSL（领域专用语言）](#175-dsl领域专用语言)
    - [17.6. 可变参数接口](#176-可变参数接口)
  - [18. 错误处理](#18-错误处理)
    - [18.1. panic](#181-panic)
    - [18.2. Option 和 unwrap](#182-option-和-unwrap)
    - [18.3. 使用 ? 解开 Option](#183-使用--解开-option)
    - [18.4. 组合算子：map](#184-组合算子map)
    - [18.5. 组合算子：and\_then](#185-组合算子and_then)
    - [18.6. 结果 Result](#186-结果-result)
    - [18.7. Result 的 map](#187-result-的-map)
    - [18.8. 给 Result 取别名](#188-给-result-取别名)
    - [18.9. 提前返回](#189-提前返回)
    - [18.10. 引入 ?](#1810-引入-)
    - [18.11. try! 宏](#1811-try-宏)
    - [18.12. 处理多种错误类型](#1812-处理多种错误类型)
    - [18.13. 从 Option 中取出 Result](#1813-从-option-中取出-result)
    - [18.14. 定义一个错误类型](#1814-定义一个错误类型)
    - [18.15. 把错误 “装箱”](#1815-把错误-装箱)
    - [18.16. ? 的其他用法](#1816--的其他用法)
    - [18.17. 包裹错误](#1817-包裹错误)
    - [18.18. 遍历Result](#1818-遍历result)
      - [18.18.1. 使用 filter\_map() 忽略失败的项](#18181-使用-filter_map-忽略失败的项)
      - [18.18.2. 使用 collect() 使整个操作失败](#18182-使用-collect-使整个操作失败)
      - [18.18.3. 使用 Partition() 收集所有合法的值与错误](#18183-使用-partition-收集所有合法的值与错误)
  - [x. 模块管理](#x-模块管理)
    - [x.1. 包和 Crate](#x1-包和-crate)
    - [x.2 定义模块来控制作用域与私有性](#x2-定义模块来控制作用域与私有性)
    - [x.3 库 Package](#x3-库-package)
      - [x.3.1. Package 结构](#x31-package-结构)

## 0. 仓库结构

```text
rust-learn/
├── README.md        本学习笔记
├── docs/images/     笔记中使用的图片
├── examples/        按主题分组的小示例，每个目录都是独立的 Cargo 项目
└── projects/        完整的综合项目
```

### 0.1. 示例（examples）

| 目录 | 主题 | 示例 | 相关笔记 |
| ---- | ---- | ---- | -------- |
| [`01-basics`](examples/01-basics) | 基础：格式化输出、枚举与 `impl` | [`format-demo`](examples/01-basics/format-demo)、[`impl-demo`](examples/01-basics/impl-demo)、[`type-demo`](examples/01-basics/type-demo) | [3. 基础语法](#3-基础语法)、[16. 方法](#16-方法-method) |
| [`02-collections`](examples/02-collections) | 集合与链表 | [`vec-demo`](examples/02-collections/vec-demo)、[`hashmap-demo`](examples/02-collections/hashmap-demo)、[`hashset-demo`](examples/02-collections/hashset-demo)、[`lists`](examples/02-collections/lists)、[`linked_list`](examples/02-collections/linked_list)、[`reverse_list_demo`](examples/02-collections/reverse_list_demo) | [13. Vector](#13-动态数组-vector)、[14. HashMap](#14-hashmap) |
| [`03-modules`](examples/03-modules) | 模块与库 | [`mod-demo`](examples/03-modules/mod-demo)、[`module-demo`](examples/03-modules/module-demo)、[`my-lib`](examples/03-modules/my-lib) | [x. 模块管理](#x-模块管理) |
| [`04-errors`](examples/04-errors) | 错误处理：`Result`、自定义错误 | [`error_demo`](examples/04-errors/error_demo)、[`result-demo`](examples/04-errors/result-demo) | [18. 错误处理](#18-错误处理) |
| [`05-generics-traits`](examples/05-generics-traits) | 泛型、特征、特征对象、生命周期 | [`genericity_demo`](examples/05-generics-traits/genericity_demo)、[`trait-demo`](examples/05-generics-traits/trait-demo)、[`trait-object`](examples/05-generics-traits/trait-object)、[`lifetime-demo`](examples/05-generics-traits/lifetime-demo) | [12. 泛型和特征](#12-泛型和特征)、[15. 生命周期](#15-生命周期) |
| [`06-smart-pointers`](examples/06-smart-pointers) | 智能指针：`Box`、`Arc` | [`mut-demo`](examples/06-smart-pointers/mut-demo)、[`rc-demo`](examples/06-smart-pointers/rc-demo) | — |
| [`07-concurrency`](examples/07-concurrency) | 并发：线程、通道、线程池 Web 服务器 | [`thread-demo`](examples/07-concurrency/thread-demo)、[`channel-demo`](examples/07-concurrency/channel-demo)、[`web-server`](examples/07-concurrency/web-server) | — |
| [`08-advanced`](examples/08-advanced) | 宏与 unsafe | [`macro_rules_demo`](examples/08-advanced/macro_rules_demo)、[`unsafe_demo`](examples/08-advanced/unsafe_demo) | [17. macro_rules!](#17-使用-macro_rules-来创建宏) |
| [`09-testing-io`](examples/09-testing-io) | 测试、文件读写、命令行参数、minigrep | [`test_demo`](examples/09-testing-io/test_demo)、[`file-demo`](examples/09-testing-io/file-demo)、[`args-demo`](examples/09-testing-io/args-demo)、[`minigrep`](examples/09-testing-io/minigrep) | — |
| [`10-web-axum`](examples/10-web-axum) | Axum Web：CORS、SSE、WebSocket 聊天、路径参数错误处理 | [`cors`](examples/10-web-axum/cors)、[`sse`](examples/10-web-axum/sse)、[`chart`](examples/10-web-axum/chart)、[`customize-path-rejection`](examples/10-web-axum/customize-path-rejection) | — |

每个示例都是独立的 Cargo 项目，进入对应目录运行即可，例如：

```sh
cd examples/07-concurrency/web-server
cargo run          # 在 http://127.0.0.1:7878 提供 hello.html，处理两个请求后退出
```

### 0.2. 综合项目（projects）

- [`smart-access-control`](projects/smart-access-control)：智能门禁系统（纯软件模拟）。
  Rust 后端（Axum、Tokio、SQLx、PostgreSQL，按 Clean Architecture 分层）+
  React 管理面板，包含通行判定引擎、JWT 登录与刷新令牌轮换、WebSocket 实时事件、
  模拟门控制器、Prometheus 指标和 Docker 部署。
  说明文档：[中文](projects/smart-access-control/README.zh-CN.md) |
  [English](projects/smart-access-control/README.md)

## 1. Rust 的安装(Mac)

```text
brew install rustup-init
```

然后执行

```text
rustup-init
```

## 2. Cargo

Cargo 是 Rust 的构建系统和包管理器。

### 2.1. 使用 Cargo 创建项目

```text
cargo new hello_cargo
```

进入 hello_cargo 目录并列出文件。将会看到 Cargo 生成了两个文件和一个目录：一个 Cargo.toml 文件，一个 src 目录，以及位于 src 目录中的 main.rs 文件。  

> 文件名: Cargo.toml: 使用 TOML (Tom's Obvious, Minimal Language) 格式，这是 Cargo 配置文件的格式。

```toml
[package]
name = "hello-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
```

第一行，[package]，是一个片段（section）标题，表明下面的语句用来配置一个包。随着我们在这个文件增加更多的信息，还将增加其他片段（section）。  

最后一行，[dependencies]，是罗列项目依赖的片段的开始。在 Rust 中，代码包被称为 crates。  

#### 2.1.1. 从 crates.io 引入依赖包

```text
[dependencies]
time = "0.1.12"
```

字符串 "0.1.12" 是一个 semver 格式的版本号，符合 "x.y.z" 的形式，其中 x 被称为主版本major, y 被称为小版本 minor ，而 z 被称为补丁 patch，从左到右，版本的影响范围逐步降低，补丁的更新是无关痛痒的，并不会造成 API 的兼容性被破坏。

"0.1.12" 中并没有任何额外的符号，在版本语义上，它跟使用了 ^ 的 "^0.1.12" 是相同的，都是指定非常具体的版本进行引入。

>| ^ 指定版本  

与之前的 "0.1.12" 不同， ^ 可以指定一个版本号范围，然后会使用该范围内的最大版本号来引用对应的包。  

```text
^1.2.3  :=  >=1.2.3, <2.0.0
^1.2    :=  >=1.2.0, <2.0.0
^1      :=  >=1.0.0, <2.0.0
^0.2.3  :=  >=0.2.3, <0.3.0
^0.2    :=  >=0.2.0, <0.3.0
^0.0.3  :=  >=0.0.3, <0.0.4
^0.0    :=  >=0.0.0, <0.1.0
^0      :=  >=0.0.0, <1.0.0
```

>| ~ 指定版本  

~ 指定了最小化版本 :

```text
~1.2.3  := >=1.2.3, <1.3.0
~1.2    := >=1.2.0, <1.3.0
~1      := >=1.0.0, <2.0.0
```

>| * 通配符

允许将 * 所在的位置替换成任何数字:

```text
*     := >=0.0.0
1.*   := >=1.0.0, <2.0.0
1.2.* := >=1.2.0, <1.3.0
```

>| 比较符  

版本号规则仅针对 crate.io 和基于它搭建的注册服务(例如科大服务源) ，其它注册服务(例如 GitHub )有自己相应的规则。  

使用比较符的方式来指定一个版本号范围或一个精确的版本号:

```text
>= 1.2.0
> 1
< 2
= 1.2.3
```

还能使用比较符进行组合，并通过逗号分隔：

```text
>= 1.2, < 1.5
```

#### 2.1.2. 从其它注册服务引入依赖包

为了使用 crates.io 之外的注册服务，需要对 $HOME/.cargo/config.toml ($CARGO_HOME 下) 文件进行配置，添加新的服务提供商，有两种方式可以实现。  

>| 使用科大的注册服务来提升下载速度

在 crates.io 之外添加新的注册服务，修改 .cargo/config.toml 添加以下内容：

```text
[registries]
ustc = { index = "https://mirrors.ustc.edu.cn/crates.io-index/" }
```

对于这种方式，项目的 Cargo.toml 中的依赖包引入方式也有所不同：

```text
[dependencies]
time = {  registry = "ustc" }
```

在重新配置后，初次构建可能要较久的时间，因为要下载更新 ustc 注册服务的索引文件，这一种使用方式最大的缺点就是在引用依赖包时要指定注册服务: time = { registry = "ustc" }。  

>| 直接使用新注册服务来替代默认的 crates.io  

将源 source.crates-io 替换为 ustc，然后在第二部分指定了 ustc 源的地址。  

注意，如果你要发布包到 crates.io 上，那该包的依赖也必须在 crates.io 上.  

```text
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```

#### 2.1.3. 引入 git 仓库作为依赖包

```text
[dependencies]
regex = { git = "https://github.com/rust-lang/regex" }
```

由于没有指定版本，Cargo 会假定我们使用 master 或 main 分支的最新 commit 。你可以使用 rev、tag 或 branch 来指定想要拉取的版本。例如下面代码拉取了 next 分支上的最新 commit：

```text
[dependencies]
regex = { git = "https://github.com/rust-lang/regex", branch = "next" }
```

任何非 tag 和 branch 的类型都可以通过 rev 来引入，例如通过最近一次 commit 的哈希值引入: rev = "4c59b707"，再比如远程仓库提供的的具名引用: rev = "refs/pull/493/head"。  

一旦 git 依赖被拉取下来，该版本就会被记录到 Cargo.lock 中进行锁定。因此 git 仓库中后续新的提交不再会被自动拉取，除非你通过 cargo update 来升级。需要注意的是锁定一旦被删除，那 Cargo 依然会按照 Cargo.toml 中配置的地址和版本去拉取新的版本，如果你配置的版本不正确，那可能会拉取下来一个不兼容的新版本！

#### 2.1.4. 通过路径引入本地依赖包

本地依赖包都是同一个项目内的内部包，例如假设我们有一个 hello_world 项目( package )，现在在其根目录下新建一个包:

```text
#  在 hello_world/ 目录下
$ cargo new hello_utils
```

新建的 hello_utils 文件夹跟 src、Cargo.toml 同级，现在修改 Cargo.toml 让 hello_world 项目引入新建的包:

```text
[dependencies]
hello_utils = { path = "hello_utils" }
# 以下路径也可以
# hello_utils = { path = "./hello_utils" }
# hello_utils = { path = "../hello_world/hello_utils" }
```

#### 2.1.5. 根据平台引入依赖

根据特定的平台来引入依赖:

```text
[target.'cfg(windows)'.dependencies]
winhttp = "0.4.0"

[target.'cfg(unix)'.dependencies]
openssl = "1.0.1"

[target.'cfg(target_arch = "x86")'.dependencies]
native = { path = "native/i686" }

[target.'cfg(target_arch = "x86_64")'.dependencies]
native = { path = "native/x86_64" }
```

还能使用逻辑操作符进行控制，当不是 unix 操作系统时，才对 openssl 进行引入。

```text
[target.'cfg(not(unix))'.dependencies]
openssl = "1.0.1"
```

#### 2.1.6. [dev-dependencies]

为项目添加只在测试时需要的依赖库，类似于 package.json( Nodejs )文件中的 devDependencies，可以在 Cargo.toml 中添加 [dev-dependencies] 来实现:

```text
[dev-dependencies]
tempdir = "0.3"
```

这里的依赖只会在运行测试、示例和 benchmark 时才会被引入。并且，假设A 包引用了 B，而 B 通过 [dev-dependencies] 的方式引用了 C 包， 那 A 是不会引用 C 包的。

还可以指定平台特定的测试依赖包:

```text
[target.'cfg(unix)'.dev-dependencies]
mio = "0.0.1"
```

#### 2.1.7. [build-dependencies]

指定某些依赖仅用于构建脚本:

```text
[build-dependencies]
cc = "1.0.3"
```

平台特定的依赖包：

[target.'cfg(unix)'.build-dependencies]
cc = "1.0.3"

#### 2.1.8 选择 features

如果依赖包提供了条件性的 features，可以指定使用哪一个:

```text
[dependencies.awesome]
version = "1.3.5"
default-features = false # 不要包含默认的 features，而是通过下面的方式来指定
features = ["secure-password", "civet"]
```

#### 2.1.9. 在 Cargo.toml 中重命名依赖

避免在 Rust 代码中使用 use foo as bar  
依赖某个包的多个版本  
依赖来自于不同注册服务的同名包  

使用 Cargo 提供的 package key :

```text
[package]
name = "mypackage"
version = "0.0.1"

[dependencies]
foo = "0.1"
bar = { git = "https://github.com/example/project", package = "foo" }
baz = { version = "0.1", registry = "custom", package = "foo" }
```

### 2.2. 构建并运行 Cargo 项目

> cargo build: 会创建一个可执行文件 target/debug/hello_rust

首次运行 cargo build 时，也会使 Cargo 在项目根目录创建一个新文件：Cargo.lock。  

> cargo run 在一个命令中同时编译并运行生成的可执行文件，比起要记得运行 cargo build 之后再用可执行文件的完整路径来运行程序，使用 cargo run 可以实现完全相同的效果，而且要方便得多，所以大多数开发者会使用 cargo run。  

> cargo check 的命令。该命令快速检查代码确保其可以编译，但并不产生可执行文件.

### 2.3. Cargo.toml 格式

Cargo.toml 又被称为清单( manifest )，文件格式是 TOML，每一个清单文件都由以下部分组成：

- cargo-features — 只能用于 nightly版本的 feature  
- [package] — 定义项目( package )的元信息  
  + name — 名称  
  + version — 版本  
  + authors — 开发作者  
  + edition — Rust edition.  
  + rust-version — 支持的最小化 Rust 版本  
  + description — 描述  
  + documentation — 文档 URL  
  + readme — README 文件的路径  
  + homepage - 主页 URL  
  + repository — 源代码仓库的 URL  
  + license — 开源协议 License.  
  + license-file — License 文件的路径.  
  + keywords — 项目的关键词  
  + categories — 项目分类  
  + workspace — 工作空间 workspace 的路径  
  + build — 构建脚本的路径  
  + links — 本地链接库的名称  
  + exclude — 发布时排除的文件  
  + include — 发布时包含的文件  
  + publish — 用于阻止项目的发布  
  + metadata — 额外的配置信息，用于提供给外部工具  
  + default-run — [cargo run] 所使用的默认可执行文件( binary )  
  + autobins — 禁止可执行文件的自动发现  
  + autoexamples — 禁止示例文件的自动发现  
  + autotests — 禁止测试文件的自动发现  
  + autobenches — 禁止 bench 文件的自动发现  
  + resolver — 设置依赖解析器( dependency resolver)  
- Cargo Target 列表: (查看 Target 配置 获取详细设置)  
  + [lib] — Library target 设置.  
  + [[bin]] — Binary target 设置.  
  + [[example]] — Example target 设置.  
  + [[test]] — Test target 设置.  
  + [[bench]] — Benchmark target 设置.  
- Dependency tables:  
  + [dependencies] — 项目依赖包  
  + [dev-dependencies] — 用于 examples、tests 和 benchmarks 的依赖包  
  + [build-dependencies] — 用于构建脚本的依赖包  
  + [target] — 平台特定的依赖包  
  + [badges] — 用于在注册服务(例如 crates.io ) 上显示项目的一些状态信息，例如当前的维护状态：活跃中、寻找维护者、deprecated  
  + [features] — features 可以用于条件编译  
  + [patch] — 推荐使用的依赖覆盖方式  
  + [replace] — 不推荐使用的依赖覆盖方式 (deprecated).  
  + [profile] — 编译器设置和优化  
  + [workspace] — 工作空间的定义  

### 2.4. 发布（release）构建

运行 cargo build --release 并使用 target/release 下的可执行文件进行测试。  

## 3. 基础语法

### 3.1. 变量

在变量名前添加 mut 来使其可变.

```rs
fn main() {
    let mut x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");
}
```

### 3.2. 常量

常量(constants) 是绑定到一个名称的不允许改变的值，不允许对常量使用 mut。

```rs
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

### 3.3. 隐藏

可以用相同变量名称来隐藏一个变量，以及重复使用 let 关键字来多次隐藏.

```rs
fn main() {
  let x = 5;
  let x = x + 1;
  {
    let x = x * 2;
    println!("The value of x in the inner scope is: {x}");
  }
  println!("The value of x is: {x}");
}
```

mut 与隐藏的另一个区别是，当再次使用 let 时，实际上创建了一个新变量，可以改变值的类型，并且复用这个名字。  

## 4. 数据类型

### 4.1. 标量类型

标量（scalar）类型代表一个单独的值。Rust 有四种基本的标量类型：整型、浮点型、布尔类型和字符类型。

#### 4.1.1. 整型

没有小数部分的数字.  

| 长度	| 有符号 | 无符号 |
|  ----  | ----  |
| 8-bit |  i8 | u8 |
| 16-bit | i16 | u16 |
| 32-bit | i32 | u32 |
| 64-bit | i64 | u64 |
| 128-bit | i128 | u128 |
| arch | isize | usize |

有符号 和 无符号 代表数字能否为负值.  

每一个有符号的变体可以储存包含从 -2^(n-1)到 2^(n-1) - 1 在内的数字，这里 n 是变体使用的位数。所以 i8 可以储存从 -2^7 到 2^7 - 1 在内的数字，也就是从 -128 到 127。无符号的变体可以储存从 0 到 2^n - 1 的数字，所以 u8 可以储存从 0 到 2^8 - 1 的数字，也就是从 0 到 255。  

isize 和 usize 类型依赖运行程序的计算机架构：64 位架构上它们是 64 位的， 32 位架构上它们是 32 位的。  

| 数字字面值 | 例子 |
| Decimal (十进制) | 98_222 |
| Hex (十六进制) | 0xff |
| Octal (八进制) | 0o77 |
| Binary (二进制) | 0b1111_0000 |
| Byte (单字节字符)(仅限于u8) | b'A' |

如果拿不定主意，Rust 的默认类型通常是个不错的起点，数字类型默认是 i32。isize 或 usize 主要作为某些集合的索引。  

> 整型溢出

比方说有一个 u8 ，它可以存放从零到 255 的值。那么当你将其修改为 256 时会发生什么呢？这被称为 “整型溢出”（“integer overflow” ），这会导致以下两种行为之一的发生。当在 debug 模式编译时，Rust 检查这类问题并使程序 panic，这个术语被 Rust 用来表明程序因错误而退出。  

在 release 构建中，Rust 不检测溢出，相反会进行一种被称为二进制补码包装（two’s complement wrapping）的操作。简而言之，值 256 变成 0，值 257 变成 1，依此类推。依赖整型溢出被认为是一种错误，即便可能出现这种行为。如果你确实需要这种行为，标准库中有一个类型显式提供此功能，Wrapping。 为了显式地处理溢出的可能性，你可以使用标准库在原生数值类型上提供的以下方法:  

所有模式下都可以使用 wrapping_* 方法进行包装，如 wrapping_add  
如果 checked_* 方法出现溢出，则返回 None值  
用 overflowing_* 方法返回值和一个布尔值，表示是否出现溢出  
用 saturating_* 方法在值的最小值或最大值处进行饱和处理  

#### 4.1.2. 浮点型

rust 的浮点数类型是 f32 和 f64，分别占 32 位和 64 位。默认类型是 f64，因为在现代 CPU 中，它与 f32 速度几乎一样，不过精度更高。所有的浮点型都是有符号的。  

### 4.2. 数值运算

加法、减法、乘法、除法和取余。整数除法会向下舍入到最接近的整数。  

```rs
fn main() {
    // addition
    let sum = 5 + 10;

    // subtraction
    let difference = 95.5 - 4.3;

    // multiplication
    let product = 4 * 30;

    // division
    let quotient = 56.7 / 32.2;
    let floored = 2 / 3; // Results in 0

    // remainder
    let remainder = 43 % 5;
}
```

### 4.3. 布尔型

Rust 中的布尔类型有两个可能的值：true 和 false。Rust 中的布尔类型使用 bool 表示。  

```rs
fn main() {
    let t = true;
    let f: bool = false;
}
```

### 4.4. 字符类型

用单引号声明 char 字面量，使用双引号声明字符串字面量。

### 4.5. 复合类型

复合类型（Compound types）可以将多个值组合成一个类型。Rust 有两个原生的复合类型：元组（tuple）和数组（array）。

#### 4.5.1. 元组类型

元组是一个将多个其他类型的值组合进一个复合类型的主要方式。元组长度固定：一旦声明，其长度不会增大或缩小。  

```rs
fn main() {
  let tup: (i32, f64, u8) = (500, 6.4, 1);
}
```

可以使用模式匹配（pattern matching）来解构（destructure）元组值

```rs
fn main() {
  let tup = (500, 6.4, 1);
  let (x, y, z) = tup;
  println!("The value of y is: {y}");
}
```

可以使用点号（.）后跟值的索引来直接访问.

```rs
fn main() {
  let x: (i32, f64, u8) = (500, 6.4, 1);
  let five_hundred = x.0;
  let six_point_four = x.1;
  let one = x.2;
}
```

不带任何值的元组有个特殊的名称，叫做 单元（unit） 元组。这种值以及对应的类型都写作 ()，表示空值或空的返回类型。如果表达式不返回任何其他值，则会隐式返回单元值。

#### 4.5.2. 数组类型

与元组不同，数组中的每个元素的类型必须相同。Rust中的数组长度是固定的。  

数组并不如 vector 类型灵活。vector 类型是标准库提供的一个 允许 增长和缩小长度的类似数组的集合类型。当不确定是应该使用数组还是 vector 的时候，那么很可能应该使用 vector。  

```rs
let a: [i32; 5] = [1, 2, 3, 4, 5];
```

这里，i32 是每个元素的类型。分号之后，数字 5 表明该数组包含五个元素。

还可以通过在方括号中指定初始值加分号再加元素个数的方式来创建一个每个元素都为相同值的数组：

```rs
let a = [3; 5];
```

变量名为 a 的数组将包含 5 个元素，这些元素的值最初都将被设置为 3。这种写法与 let a = [3, 3, 3, 3, 3]; 效果相同，但更简洁。

> 访问数组元素

使用索引来访问数组的元素

```rs
fn main() {
  let a = [1, 2, 3, 4, 5];

  let first = a[0];
  let second = a[1];
}
```

## 5. 函数

Rust 代码中的函数和变量名使用 snake case 规范风格。在 snake case 中，所有字母都是小写并使用下划线分隔单词。

```rs
fn main() {
  println!("Hello, world!");
  another_function();
}

fn another_function() {
  println!("Another function.");
}
```

Rust 不关心函数定义所在的位置，只要函数被调用时出现在调用之处可见的作用域内就行。  

### 5.1. 参数

在函数签名中，必须 声明每个参数的类型。

```rs
fn main() {
  print_labeled_measurement(5, 'h');
}

fn print_labeled_measurement(value: i32, unit_label: char) {
  println!("The measurement is: {value}{unit_label}");
}
```

### 5.2. 语句和表达式

> 语句（Statements）是执行一些操作但不返回值的指令。  

let y = 6; 是一个语句。  

```rs
fn main() {
  let y = 6;
}
```

> 表达式（Expressions）计算并产生一个值。

函数调用是一个表达式。宏调用是一个表达式。用大括号创建的一个新的块作用域也是一个表达式.

```rs
fn main() {
  let y = {
    let x = 3;
    x + 1
  };

  println!("The value of y is: {y}");
}
```

表达式是一个代码块，它的值是 4。

```rs
{
  let x = 3;
  x + 1
}
```

### 5.3. 函数的返回值

要在箭头（->）后声明它的类型.  
函数的返回值等同于函数体最后一个表达式的值。使用 return 关键字和指定值，可从函数中提前返回；但大部分函数隐式的返回最后的表达式.

```rs
fn main() {
    let x = plus_one(5);

    println!("The value of x is: {x}");
}

fn plus_one(x: i32) -> i32 {
    x + 1
}
```

运行代码会打印出 The value of x is: 6。但如果在包含 x + 1 的行尾加上一个分号，把它从表达式变成语句，将看到一个错误。

```rs
fn main() {
  let x = plus_one(5);

  println!("The value of x is: {x}");
}

fn plus_one(x: i32) -> i32 {
  x + 1;
}
```

```text
7 | fn plus_one(x: i32) -> i32 {
  |    --------            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
8 |     x + 1;
  |          - help: consider removing this semicolon

For more information about this error, try `rustc --explain E0308`.
error: could not compile `functions` due to previous error
```

“mismatched types”（类型不匹配），揭示了代码的核心问题。函数 plus_one 的定义说明它要返回一个 i32 类型的值，不过语句并不会返回值，使用单位类型 () 表示不返回值。因为不返回值与函数定义相矛盾，从而出现一个错误。在输出中，Rust 提供了一条信息，可能有助于纠正这个错误：它建议删除分号，这会修复这个错误。

## 6. 控制流

Rust 代码中最常见的用来控制执行流的结构是 if 表达式和循环。  

### 6.1. if 表达式

```rs
fn main() {
  let number = 3;

  if number < 5 {
    println!("condition was true");
  } else {
    println!("condition was false");
  }
}
```

> 在 let 语句中使用 if

```rs
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 };

    println!("The value of number is: {number}"); // 5
}
```

### 6.2. 使用循环重复执行

Rust 有三种循环：loop、while 和 for。

#### 6.2.1. 使用 loop 重复执行代码

可以使用 break 关键字来告诉程序何时停止循环。  
循环中的 continue 关键字告诉程序跳过这个循环迭代中的任何剩余代码，并转到下一个迭代。  

> 循环返回值

```rs
fn main() {
  let mut counter = 0;
  let result = loop {
      counter += 1;
      if counter == 10 {
          break counter * 2;
      }
  };
  println!("The result is {result}"); // 20
}
```

> 循环标签：在多个循环之间消除歧义

如果存在嵌套循环，break 和 continue 应用于此时最内层的循环。可以选择在一个循环上指定一个 循环标签（loop label），然后将标签与 break 或 continue 一起使用，使这些关键字应用于已标记的循环, 而不是最内层的循环。

```rs
fn main() {
    let mut count = 0;
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break;
            }
            if count == 2 {
                break 'counting_up;
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}"); // 2
}
```

#### 6.2.2. while 条件循环

```rs
fn main() {
  let mut number = 3;

  while number != 0 {
    println!("{number}!");

    number -= 1;
  }

  println!("LIFTOFF!!!");
}
```

#### 6.2.3. 使用 for 遍历集合

```rs
fn main() {
  let a = [10, 20, 30, 40, 50];

  for element in a {
      println!("the value is: {element}");
  }
}
```

## 7. 所有权

所有程序都必须管理其运行时使用计算机内存的方式。一些语言中具有垃圾回收机制，在程序运行时不断地寻找不再使用的内存；在另一些语言中，程序员必须亲自分配和释放内存。Rust 则选择了第三种方式：通过所有权系统管理内存，编译器在编译时会根据一系列的规则进行检查。如果违反了任何这些规则，程序都不能编译。在运行时，所有权系统的任何功能都不会减慢程序。

> 所有权规则

1. Rust 中的每一个值都有一个被称为其 所有者（owner）的变量。
2. 值在任一时刻有且只有一个所有者。
3. 当所有者（变量）离开作用域，这个值将被丢弃。

> 变量作用域

作用域（scope）是一个项（item）在程序中有效的范围。

> 内存与分配

内存在拥有它的变量离开作用域后就被自动释放。

1. 变量与数据交互的方式（一）：移动

```rs
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;

    println!("{}, world!", s1);
}
```

为了确保内存安全，在 let s2 = s1 之后，Rust 认为 s1 不再有效，因此 Rust 不需要在 s1 离开作用域后清理任何东西。

```text
 --> src/main.rs:5:28
  |
2 |     let s1 = String::from("hello");
  |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
3 |     let s2 = s1;
  |              -- value moved here
4 | 
5 |     println!("{}, world!", s1);
  |                            ^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `ownership` due to previous error
```

在其他语言中听说过术语 浅拷贝（shallow copy）和 深拷贝（deep copy），那么拷贝指针、长度和容量而不拷贝数据可能听起来像浅拷贝。不过因为 Rust 同时使第一个变量无效了，这个操作被称为 移动（move），而不是浅拷贝。  

Rust 永远也不会自动创建数据的 “深拷贝”。因此，任何 自动 的复制可以被认为对运行时性能影响较小。

2. 变量与数据交互的方式（二）：克隆

```rs
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();

    println!("s1 = {}, s2 = {}", s1, s2);
}
```

3. 只在栈上的数据：拷贝

```rs
fn main() {
    let x = 5;
    let y = x;

    println!("x = {}, y = {}", x, y);
}
```

一个通用的规则，任何一组简单标量值的组合都可以实现 Copy，任何不需要分配内存或某种形式资源的类型都可以实现 Copy 。如下是一些 Copy 的类型：

- 所有整数类型，比如 u32。
- 布尔类型，bool，它的值是 true 和 false。
- 所有浮点数类型，比如 f64。
- 字符类型，char。
- 元组，当且仅当其包含的类型也都实现 Copy 的时候。比如，(i32, i32) 实现了 Copy，但 (i32, String) 就没有。

> 所有权与函数

```rs
fn main() {
    let s = String::from("hello");  // s 进入作用域

    takes_ownership(s);             // s 的值移动到函数里 ...
                                    // ... 所以到这里不再有效

    let x = 5;                      // x 进入作用域

    makes_copy(x);                  // x 应该移动函数里，
                                    // 但 i32 是 Copy 的，
                                    // 所以在后面可继续使用 x

} // 这里, x 先移出了作用域，然后是 s。但因为 s 的值已被移走，
  // 没有特殊之处

fn takes_ownership(some_string: String) { // some_string 进入作用域
    println!("{}", some_string);
} // 这里，some_string 移出作用域并调用 `drop` 方法。
  // 占用的内存被释放

fn makes_copy(some_integer: i32) { // some_integer 进入作用域
    println!("{}", some_integer);
} // 这里，some_integer 移出作用域。没有特殊之处
```

> 返回值与作用域

返回值也可以转移所有权。

```rs
fn main() {
    let s1 = gives_ownership();         // gives_ownership 将返回值
                                        // 转移给 s1

    let s2 = String::from("hello");     // s2 进入作用域

    let s3 = takes_and_gives_back(s2);  // s2 被移动到
                                        // takes_and_gives_back 中,
                                        // 它也将返回值移给 s3
} // 这里, s3 移出作用域并被丢弃。s2 也移出作用域，但已被移走，
  // 所以什么也不会发生。s1 离开作用域并被丢弃

fn gives_ownership() -> String {             // gives_ownership 会将
                                             // 返回值移动给
                                             // 调用它的函数

    let some_string = String::from("yours"); // some_string 进入作用域.

    some_string                              // 返回 some_string 
                                             // 并移出给调用的函数
                                             // 
}

// takes_and_gives_back 将传入字符串并返回该值
fn takes_and_gives_back(a_string: String) -> String { // a_string 进入作用域

    a_string  // 返回 a_string 并移出给调用的函数
}
```

### 7.1. 引用与借用

> 引用

引用（reference）像一个指针，因为它是一个地址，可以由此访问储存于该地址的属于其他变量的数据。与指针不同，引用确保指向某个特定类型的有效值。

```rs
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{}' is {}.", s1, len);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

& 符号就是 引用，它们允许你使用值但不获取其所有权。

![引用](/docs/images/1.svg)

&s1 语法让我们创建一个 指向 值 s1 的引用，但是并不拥有它。因为并不拥有这个值，所以当引用停止使用时，它所指向的值也不会被丢弃。

```rs
let s1 = String::from("hello");
let len = calculate_length(&s1);
```

> 借用

将创建一个引用的行为称为 借用（borrowing）。

```rs
fn main() {
    let s = String::from("hello");

    change(&s);
}

fn change(some_string: &String) {
    some_string.push_str(", world");
}
```

不允许修改引用的值。

```text
 --> src/main.rs:8:5
  |
7 | fn change(some_string: &String) {
  |                        ------- help: consider changing this to be a mutable reference: `&mut String`
8 |     some_string.push_str(", world");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `some_string` is a `&` reference, so the data it refers to cannot be borrowed as mutable

For more information about this error, try `rustc --explain E0596`.
error: could not compile `ownership` due to previous error
```

> 可变引用

允许我们修改一个借用的值，这就是 可变引用（mutable reference）：

修复上述代码的错误: 

```rs
fn main() {
  let mut s = String::from("hello");
  change(&mut s);
}

fn change(some_string: &mut String) {
  some_string.push_str(", world");
}
```

将 s 改为 mut。然后在调用 change 函数的地方创建一个可变引用 &mut s，并更新函数签名以接受一个可变引用 some_string: &mut String。这就非常清楚地表明，change 函数将改变它所借用的值。

可变引用有一个很大的限制：在同一时间只能有一个对某一特定数据的可变引用。这些尝试创建两个 s 的可变引用的代码会失败：

```rs
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;

    println!("{}, {}", r1, r2);
}
```

报错：

```text
4 |     let r1 = &mut s;
  |              ------ first mutable borrow occurs here
5 |     let r2 = &mut s;
  |              ^^^^^^ second mutable borrow occurs here
6 | 
7 |     println!("{}, {}", r1, r2);
  |                        -- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
error: could not compile `ownership` due to previous error
```

可以使用大括号来创建一个新的作用域，以允许拥有多个可变引用，只是不能 同时 拥有：

```rs
fn main() {
    let mut s = String::from("hello");
    {
        let r1 = &mut s;
    } // r1 在这里离开了作用域，所以我们完全可以创建一个新的引用
    let r2 = &mut s;
}
```

不能在拥有不可变引用的同时拥有可变引用。不可变引用的用户可不希望在他们的眼皮底下值就被意外的改变了！然而，多个不可变引用是可以的，因为没有哪个只能读取数据的人有能力影响其他人读取到的数据。

```rs
fn main() {
  let mut s = String::from("hello");

  let r1 = &s; // 没问题
  let r2 = &s; // 没问题
  let r3 = &mut s; // 大问题

  println!("{}, {}, and {}", r1, r2, r3);
}
```

一个引用的作用域从声明的地方开始一直持续到最后一次使用为止。例如，因为最后一次使用不可变引用（println!)，发生在声明可变引用之前，所以如下代码是可以编译的：

```rs
fn main() {
  let mut s = String::from("hello");

  let r1 = &s; // 没问题
  let r2 = &s; // 没问题
  println!("{} and {}", r1, r2);
  // 此位置之后 r1 和 r2 不再使用

  let r3 = &mut s; // 没问题
  println!("{}", r3);
}
```

> 悬垂引用（Dangling References）

悬垂指针是其指向的内存可能已经被分配给其它持有者。相比之下，在 Rust 中编译器确保引用永远也不会变成悬垂状态：当你拥有一些数据的引用，编译器确保数据不会在其引用之前离开作用域。  

```rs
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String {
    let s = String::from("hello");

    &s
}
```

错误：

```text
 --> src/main.rs:5:16
  |
5 | fn dangle() -> &String {
  |                ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: consider using the `'static` lifetime
  |
5 | fn dangle() -> &'static String {
  |                ~~~~~~~~

For more information about this error, try `rustc --explain E0106`.
error: could not compile `ownership` due to previous error
```

解决方法是直接返回 String：

```rs
fn no_dangle() -> String {
    let s = String::from("hello");

    s
}
```

### 7.2. Slice 类型

字符串 slice（string slice）是 String 中一部分值的引用

```rs
let s = String::from("hello world");
let hello = &s[0..5];
let world = &s[6..11];
```

使用一个由中括号中的 [starting_index..ending_index] 指定的 range 创建一个 slice，其中 starting_index 是 slice 的第一个位置，ending_index 则是 slice 最后一个位置的后一个值。在其内部，slice 的数据结构存储了 slice 的开始位置和长度，长度对应于 ending_index 减去 starting_index 的值。所以对于 let world = &s[6..11]; 的情况，world 将是一个包含指向 s 索引 6 的指针和长度值 5 的 slice。

对于 Rust 的 .. range 语法，如果想要从索引 0 开始，可以不写两个点号之前的值。如下两个语句是相同的：

```rs
let s = String::from("hello");

let slice = &s[0..2];
let slice = &s[..2];
```

如果 slice 包含 String 的最后一个字节，也可以舍弃尾部的数字。

```rs
let s = String::from("hello");

let len = s.len();

let slice = &s[3..len];
let slice = &s[3..];
```

同时舍弃这两个值来获取整个字符串的 slice。

```rs
let slice = &s[0..len];
let slice = &s[..];
```

定义一个获取字符串 slice 而不是 String 引用的函数使得我们的 API 更加通用并且不会丢失任何功能：

```rs
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}

fn main() {
  let my_string = String::from("hello world");

  // `first_word` 适用于 `String`（的 slice），整体或全部
  let word = first_word(&my_string[0..6]);
  let word = first_word(&my_string[..]);
  // `first_word` 也适用于 `String` 的引用，
  // 这等价于整个 `String` 的 slice
  let word = first_word(&my_string);

  let my_string_literal = "hello world";

  // `first_word` 适用于字符串字面值，整体或全部
  let word = first_word(&my_string_literal[0..6]);
  let word = first_word(&my_string_literal[..]);

  // 因为字符串字面值已经 **是** 字符串 slice 了，
  // 这也是适用的，无需 slice 语法！
  let word = first_word(my_string_literal);
}
```

## 8. 结构体

结构体需要命名各部分数据以便能清楚的表明其值的意义。由于有了这些名字，结构体比元组更灵活：不需要依赖顺序来指定或访问实例中的值。

存储用户账号信息的结构体：

```rs
struct User {
  active: bool,
  username: String,
  email: String,
  sign_in_count: u64,
}
```

使用: 通过为每个字段指定具体值来创建这个结构体的实例。  

创建一个实例需要以结构体的名字开头，接着在大括号中使用 key: value 键-值对的形式提供字段.  

声明一个特定的用户：

```rs
struct User {
  active: bool,
  username: String,
  email: String,
  sign_in_count: u64,
}

fn main() {
  let user1 = User {
    email: String::from("someone@example.com"),
    username: String::from("someusername123"),
    active: true,
    sign_in_count: 1,
  };
}
```

为了从结构体中获取某个特定的值，可以使用点号。想要用户的邮箱地址，可以用 user1.email。  
要更改结构体中的值，如果结构体的实例是可变的，可以使用点号并为对应的字段赋值。  

```rs
fn main() {
  let mut user1 = User {
    email: String::from("someone@example.com"),
    username: String::from("someusername123"),
    active: true,
    sign_in_count: 1,
  };

  user1.email = String::from("anotheremail@example.com");
}
```

build_user 函数，它返回一个带有给定的 email 和用户名的 User 结构体实例。

```rs
fn build_user(email: String, username: String) -> User {
  User {
    email: email,
    username: username,
    active: true,
    sign_in_count: 1,
  }
}

// 简洁写法
fn build_user(email: String, username: String) -> User {
  User {
    email,
    username,
    active: true,
    sign_in_count: 1,
  }
}
```

.. 语法指定了剩余未显式设置值的字段应有与给定实例对应字段相同的值。

```rs
let user1 = User {
  email: String::from("someone@example.com"),
  username: String::from("someusername123"),
  active: true,
  sign_in_count: 1,
};

let user2 = User {
  active: user1.active,
  username: user1.username,
  email: String::from("another@example.com"),
  sign_in_count: user1.sign_in_count,
};

// ..语法
let user2 = User {
  email: String::from("another@example.com"),
  ..user1
};
```

> 定义元组结构体

以 struct 关键字和结构体名开头并后跟元组中的类型.  

```rs
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
  let black = Color(0, 0, 0);
  let origin = Point(0, 0, 0);
}
```

> 没有任何字段的类单元结构体

被称为 类单元结构体（unit-like structs）因为它们类似于 ()，即“元组类型”一节中提到的 unit 类型。

```rs
struct AlwaysEqual;

fn main() {
  let subject = AlwaysEqual;
}
```

### 8.1. 方法

方法（method）与函数类似：使用 fn 关键字和名称声明，可以拥有参数和返回值，同时包含在某处调用该方法时会执行的代码。不过方法与函数是不同的，因为它们在结构体的上下文中被定义（或者是枚举或 trait 对象的上下文），并且第一个参数总是 self，它代表调用该方法的结构体实例。

在 Rectangle 结构体上定义 area 方法

```rs
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };
    let rect3 = Rectangle {
        width: 60,
        height: 45,
    };

    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
}
```

在 area 的签名中，使用 &self 来替代 rectangle: &Rectangle，&self 实际上是 self: &Self 的缩写。  

self 前面使用 & 来表示这个方法借用了 Self 实例.  

## 9. 枚举

```rs
fn main() {
  enum IpAddrKind {
    V4,
    V6,
  }

  struct IpAddr {
    kind: IpAddrKind,
    address: String,
  }

  let home = IpAddr {
    kind: IpAddrKind::V4,
    address: String::from("127.0.0.1"),
  };

  let loopback = IpAddr {
    kind: IpAddrKind::V6,
    address: String::from("::1"),
  };
}
```

一个 Message 枚举，其每个成员都存储了不同数量和类型的值

```rs
enum Message {
  Quit,
  Move { x: i32, y: i32 },
  Write(String),
  ChangeColor(i32, i32, i32),
}
```

- Quit 没有关联任何数据。
- Move 类似结构体包含命名字段。
- Write 包含单独一个 String。
- ChangeColor 包含三个 i32。

## 10. match 控制流

将一个值与一系列的模式相比较，并根据相匹配的模式执行相应代码。模式可由字面值、变量、通配符和许多其他内容构成.  

```rs
enum Coin {
  Penny,
  Nickel,
  Dime,
  Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
  match coin {
    Coin::Penny => {
      println!("Lucky penny!");
      1
    }
    Coin::Nickel => 5,
    Coin::Dime => 10,
    Coin::Quarter => 25,
  }
}
```

## 11. if let 控制流

if let 语法获取通过等号分隔的一个模式和一个表达式。它的工作方式与 match 相同，这里的表达式对应 match 而模式则对应第一个分支

```rs
let config_max = Some(3u8);
match config_max {
  Some(max) => println!("The maximum is configured to be {}", max),
  _ => (),
}

// 功能同上
let config_max = Some(3u8);
if let Some(max) = config_max {
   println!("The maximum is configured to be {}", max);
}
```

## 12. 泛型和特征

泛型参数的名称你可以任意起，但是出于惯例，我们都用 T ( T 是 type 的首字母)来作为首选。  

使用泛型参数，必需在使用前对其进行声明：

```rust
fn largest<T>(list: &[T]) -> T {
```

### 12.1. 结构体中使用泛型

结构体中的字段类型也可以用泛型来定义, 有两点需要特别的注意：  

- 提前声明，跟泛型函数定义类似，首先我们在使用泛型参数之前必需要进行声明 Point<T>，接着就可以在结构体的字段类型中使用 T 来替代具体的类型
- x 和 y 是相同的类型

```rust
struct Point<T> {
    x: T,
    y: T,
}

fn main() {
    let integer = Point { x: 5, y: 10 };
    let float = Point { x: 1.0, y: 4.0 };
}
```

### 12.2. 枚举中使用泛型



```rust
enum Option<T> {
    Some(T),
    None,
}
```

### 12.3. 方法中使用泛型

使用泛型参数前，依然需要提前声明：impl<T>，只有提前声明了，我们才能在Point<T>中使用它，这样 Rust 就知道 Point 的尖括号中的类型是泛型而不是具体类型。需要注意的是，这里的 Point<T> 不再是泛型声明，而是一个完整的结构体类型，因为我们定义的结构体就是 Point<T> 而不再是 Point。

```rust
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

fn main() {
    let p = Point { x: 5, y: 10 };

    println!("p.x = {}", p.x());
}
```

### 12.4. const 泛型（Rust 1.51 版本引入的重要特性）

定义了一个类型为 [T; N] 的数组，其中 T 是一个基于类型的泛型参数， N 这个泛型参数，它是一个基于值的泛型参数！因为它用来替代的是数组的长度。  

N 就是 const 泛型，定义的语法是 const N: usize，表示 const 泛型 N ，它基于的值类型是 usize。  
对 T 加一个限制 std::fmt::Debug，该限制表明 T 可以用在 println!("{:?}", arr) 中，因为 {:?} 形式的格式化输出需要 arr 实现该特征。  

```rust
fn display_array<T: std::fmt::Debug, const N: usize>(arr: [T; N]) {
    println!("{:?}", arr);
}
fn main() {
    let arr: [i32; 3] = [1, 2, 3];
    display_array(arr);

    let arr: [i32; 2] = [1, 2];
    display_array(arr);
}
```

#### 12.4.1 const 泛型表达式

假设我们某段代码需要在内存很小的平台上工作，因此需要限制函数参数占用的内存大小，此时就可以使用 const 泛型表达式来实现：

```rust
// 目前只能在nightly版本下使用
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

fn something<T>(val: T)
where
    Assert<{ core::mem::size_of::<T>() < 768 }>: IsTrue,
    //       ^-----------------------------^ 这里是一个 const 表达式，换成其它的 const 表达式也可以
{
    //
}

fn main() {
    something([0u8; 0]); // ok
    something([0u8; 512]); // ok
    something([0u8; 1024]); // 编译错误，数组长度是1024字节，超过了768字节的参数长度限制
}

// ---

pub enum Assert<const CHECK: bool> {
    //
}

pub trait IsTrue {
    //
}

impl IsTrue for Assert<true> {
    //
}
```

### 12.5. 泛型的性能

Rust 通过在编译时进行泛型代码的 单态化(monomorphization)来保证效率。单态化是一个通过填充编译时使用的具体类型，将通用代码转换为特定代码的过程。  

编译器所做的工作正好与我们创建泛型函数的步骤相反，编译器寻找所有泛型代码被调用的位置并针对具体类型生成代码。  

一个使用标准库中 Option 枚举的例子：

```rust
let integer = Some(5);
let float = Some(5.0);
```

当 Rust 编译这些代码的时候，它会进行单态化。编译器会读取传递给 Option<T> 的值并发现有两种 Option<T>：一种对应 i32 另一种对应 f64。为此，它会将泛型定义 Option<T> 展开为 Option_i32 和 Option_f64，接着将泛型定义替换为这两个具体的定义。

编译器生成的单态化版本的代码看起来像这样：

```rust
enum Option_i32 {
    Some(i32),
    None,
}

enum Option_f64 {
    Some(f64),
    None,
}

fn main() {
    let integer = Option_i32::Some(5);
    let float = Option_f64::Some(5.0);
}
```

### 12.6. 特征 Trait

如果不同的类型具有相同的行为，那么我们就可以定义一个特征，然后为这些类型实现该特征。定义特征是把一些方法组合在一起，目的是定义一个实现某些目标所必需的行为的集合。  

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}
```

这里使用 trait 关键字来声明一个特征，Summary 是特征名。在大括号中定义了该特征的所有方法，在这个例子中是： fn summarize(&self) -> String。

#### 12.6.1. 为类型实现特征

为 Post 和 Weibo 实现 Summary 特征：

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}
pub struct Post {
    pub title: String, // 标题
    pub author: String, // 作者
    pub content: String, // 内容
}

impl Summary for Post {
    fn summarize(&self) -> String {
        format!("文章{}, 作者是{}", self.title, self.author)
    }
}

pub struct Weibo {
    pub username: String,
    pub content: String
}

impl Summary for Weibo {
    fn summarize(&self) -> String {
        format!("{}发表了微博{}", self.username, self.content)
    }
}

fn main() {
    let post = Post{title: "Rust语言简介".to_string(),author: "Sunface".to_string(), content: "Rust棒极了!".to_string()};
    let weibo = Weibo{username: "sunface".to_string(),content: "好像微博没Tweet好用".to_string()};

    println!("{}",post.summarize());
    println!("{}",weibo.summarize());
}
```

#### 12.6.2. 特征定义与实现的位置(孤儿规则)

关于特征实现与定义的位置，有一条非常重要的原则：如果你想要为类型 A 实现特征 T，那么 A 或者 T 至少有一个是在当前作用域中定义的！ 例如我们可以为上面的 Post 类型实现标准库中的 Display 特征，这是因为 Post 类型定义在当前的作用域中。同时，我们也可以在当前包中为 String 类型实现 Summary 特征，因为 Summary 定义在当前作用域中。

但是你无法在当前作用域中，为 String 类型实现 Display 特征，因为它们俩都定义在标准库中，其定义所在的位置都不在当前作用域，该规则被称为孤儿规则，可以确保其它人编写的代码不会破坏你的代码。

#### 12.6.3. 默认实现

可以在特征中定义具有默认实现的方法，这样其它类型无需再实现该方法，或者也可以选择重载该方法：

```rust
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}
```

上面为 Summary 定义了一个默认实现，下面我们编写段代码来测试下：

```rust
impl Summary for Post {}

impl Summary for Weibo {
    fn summarize(&self) -> String {
        format!("{}发表了微博{}", self.username, self.content)
    }
}
```

可以看到，Post 选择了默认实现，而 Weibo 重载了该方法，调用和输出如下：

```rust
    println!("{}",post.summarize());
    println!("{}",weibo.summarize());
(Read more...)
sunface发表了微博好像微博没Tweet好用
```

#### 12.6.4. 使用特征作为函数参数

```rust
pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
```

impl Summary，的意思是 实现了Summary特征 的 item 参数。

你可以使用任何实现了 Summary 特征的类型作为该函数的参数，同时在函数体内，还可以调用该特征的方法，例如 summarize 方法。具体的说，可以传递 Post 或 Weibo 的实例来作为参数，而其它类如 String 或者 i32 的类型则不能用做该函数的参数，因为它们没有实现 Summary 特征。

#### 12.6.5. 特征约束(trait bound)

impl Trait 这种语法只是一个语法糖：

```rust
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```

真正的完整书写形式如上所述，形如 T: Summary 被称为特征约束。

在简单的场景下 impl Trait 这种语法糖就足够使用，但是对于复杂的场景，特征约束可以让我们拥有更大的灵活性和语法表现能力，例如一个函数接受两个 impl Summary 的参数：

```rust
pub fn notify(item1: &impl Summary, item2: &impl Summary) {}
```

如果函数两个参数是不同的类型，那么上面的方法很好，只要这两个类型都实现了 Summary 特征即可。但是如果我们想要强制函数的两个参数是同一类型呢？上面的语法就无法做到这种限制，此时我们只能使特征约束来实现：

```rust
pub fn notify<T: Summary>(item1: &T, item2: &T) {}
```

泛型类型 T 说明了 item1 和 item2 必须拥有同样的类型，同时 T: Summary 说明了 T 必须实现 Summary 特征。

#### 12.6.6. 多重约束

除了单个约束条件，我们还可以指定多个约束条件，例如除了让参数实现 Summary 特征外，还可以让参数实现 Display 特征以控制它的格式化输出：

```rust
pub fn notify(item: &(impl Summary + Display)) {}
```

除了上述的语法糖形式，还能使用特征约束的形式：

```rust
pub fn notify<T: Summary + Display>(item: &T) {}
```

通过这两个特征，就可以使用 item.summarize 方法，以及通过 println!("{}", item) 来格式化输出 item。

#### 12.6.7. Where 约束

当特征约束变得很多时，函数的签名将变得很复杂：

```rust
fn some_function<T: Display + Clone, U: Clone + Debug>(t: &T, u: &U) -> i32 {}
```

通过 where对其做一些形式上的改进

```rust
fn some_function<T, U>(t: &T, u: &U) -> i32
    where T: Display + Clone,
          U: Clone + Debug
{}
```

#### 12.6.8. 使用特征约束有条件地实现方法或特征

特征约束，可以让我们在指定类型 + 指定特征的条件下去实现方法，例如：

```rust
use std::fmt::Display;

struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("The largest member is x = {}", self.x);
        } else {
            println!("The largest member is y = {}", self.y);
        }
    }
}
```

cmp_display 方法，并不是所有的 Pair<T> 结构体对象都可以拥有，只有 T 同时实现了 Display + PartialOrd 的 Pair<T> 才可以拥有此方法。  
该函数可读性会更好，因为泛型参数、参数、返回值都在一起，可以快速的阅读，同时每个泛型参数的特征也在新的代码行中通过特征约束进行了约束。  

也可以有条件地实现特征, 例如，标准库为任何实现了 Display 特征的类型实现了 ToString 特征：

```rust
impl<T: Display> ToString for T {
    // --snip--
}
```

我们可以对任何实现了 Display 特征的类型调用由 ToString 定义的 to_string 方法。例如，可以将整型转换为对应的 String 值，因为整型实现了 Display：

```rust
let s = 3.to_string();
```

#### 12.6.9. 函数返回中的 impl Trait

通过 impl Trait 来说明一个函数返回了一个类型，该类型实现了某个特征：

```rust
fn returns_summarizable() -> impl Summary {
    Weibo {
        username: String::from("sunface"),
        content: String::from("m1 max太厉害了，电脑再也不会卡"),
    }
}
```

impl Trait 形式的返回值，在一种场景下非常非常有用，那就是返回的真实类型非常复杂，你不知道该怎么声明时(毕竟 Rust 要求你必须标出所有的类型)，此时就可以用 impl Trait 的方式简单返回。  

这种返回值方式有一个很大的限制：只能有一个具体的类型.  

#### 12.6.10. 为自定义类型实现 + 操作



```rust
use std::ops::Add;

// 为Point结构体派生Debug特征，用于格式化输出
#[derive(Debug)]
struct Point<T: Add<T, Output = T>> { //限制类型T必须实现了Add特征，否则无法进行+操作。
    x: T,
    y: T,
}

impl<T: Add<T, Output = T>> Add for Point<T> {
    type Output = Point<T>;

    fn add(self, p: Point<T>) -> Point<T> {
        Point{
            x: self.x + p.x,
            y: self.y + p.y,
        }
    }
}

fn add<T: Add<T, Output=T>>(a:T, b:T) -> T {
    a + b
}

fn main() {
    let p1 = Point{x: 1.1f32, y: 1.1f32};
    let p2 = Point{x: 2.1f32, y: 2.1f32};
    println!("{:?}", add(p1, p2));

    let p3 = Point{x: 1i32, y: 1i32};
    let p4 = Point{x: 2i32, y: 2i32};
    println!("{:?}", add(p3, p4));
}
```

### 12.7. 特征对象





## 13. 动态数组 Vector

动态数组只能存储相同类型的元素，如果你想存储不同类型的元素，可以使用之前讲过的枚举类型或者特征对象。

### 13.1. 创建动态数组

v 被显式地声明了类型 Vec<i32>

> Vec::new

```rust
let v: Vec<i32> = Vec::new();
```

```rust
let mut v = Vec::new();
v.push(1);
```

此时，v 就无需手动声明类型，因为编译器通过 v.push(1)，推测出 v 中的元素类型是 i32，因此推导出 v 的类型是 Vec<i32>。

如果预先知道要存储的元素个数，可以使用 Vec::with_capacity(capacity) 创建动态数组，这样可以避免因为插入大量新数据导致频繁的内存分配和拷贝，提升性能.

> vec![]

使用宏 vec! 来创建数组，与 Vec::new 有所不同，前者能在创建同时给予初始化值：

```rust
let v = vec![1, 2, 3];
```

此处的 v 也无需标注类型，编译器只需检查它内部的元素即可自动推导出 v 的类型是 Vec<i32> （Rust 中，整数默认类型是 i32，在数值类型中有详细介绍）。

### 13.2. 更新 Vector

向数组尾部添加元素，可以使用 push 方法：

```rust
let mut v = Vec::new();
v.push(1);
```

与其它类型一样，必须将 v 声明为 mut 后，才能进行修改。

### 13.3. 从 Vector 中读取元素

读取指定位置的元素有两种方式可选：

- 通过下标索引访问。
- 使用 get 方法。

```rust
let v = vec![1, 2, 3, 4, 5];

let third: &i32 = &v[2];
println!("第三个元素是 {}", third);

match v.get(2) {
    Some(third) => println!("第三个元素是 {third}"),
    None => println!("去你的第三个元素，根本没有！"),
}
```

> 集合类型的索引下标都是从 0 开始，&v[2] 表示借用 v 中的第三个元素，最终会获得该元素的引用。而 v.get(2) 也是访问第三个元素，但是有所不同的是，它返回了 Option<&T>，因此还需要额外的 match 来匹配解构出具体的值。

> 下标索引与 .get 的区别.

涉及到数组越界的问题

```rust
let v = vec![1, 2, 3, 4, 5];

let does_not_exist = &v[100];
let does_not_exist = v.get(100);
```

运行以上代码，&v[100] 的访问方式会导致程序无情报错退出，因为发生了数组越界访问。 但是 v.get 就不会，它在内部做了处理，有值的时候返回 Some(T)，无值的时候返回 None，因此 v.get 的使用方式非常安全。

### 13.4. 同时借用多个数组元素

first = &v[0] 进行了不可变借用，v.push 进行了可变借用，如果 first 在 v.push 之后不再使用，那么该段代码可以成功编译，原因是引用的作用域。

```rust
let mut v = vec![1, 2, 3, 4, 5];

let first = &v[0];

v.push(6);

println!("The first element is: {first}");
//  ----- immutable borrow later used here // 不可变借用在这里被使用
```

原因在于：数组的大小是可变的，当旧数组的大小不够用时，Rust 会重新分配一块更大的内存空间，然后把旧数组拷贝过来。这种情况下，之前的引用显然会指向一块无效的内存。

### 13.5. 迭代遍历 Vector 中的元素

使用迭代的方式去遍历数组，这种方式比用下标的方式去遍历数组更安全也更高效（每次下标访问都会触发数组边界检查）：

```rust
let v = vec![1, 2, 3];
for i in &v {
    println!("{i}");
}
```

也可以在迭代过程中，修改 Vector 中的元素：

```rust
let mut v = vec![1, 2, 3];
for i in &mut v {
    *i += 10
}
```

### 13.6. 存储不同类型的元素

通过使用枚举类型和特征对象来实现不同类型元素的存储。

> 枚举类型

```rust
#[derive(Debug)]
enum IpAddr {
    V4(String),
    V6(String)
}
fn main() {
    let v = vec![
        IpAddr::V4("127.0.0.1".to_string()),
        IpAddr::V6("::1".to_string())
    ];

    for ip in v {
        show_addr(ip)
    }
}

fn show_addr(ip: IpAddr) {
    println!("{:?}",ip);
}
```

> 特征对象

```rust
trait IpAddr {
    fn display(&self);
}

struct V4(String);
impl IpAddr for V4 {
    fn display(&self) {
        println!("ipv4: {:?}",self.0)
    }
}
struct V6(String);
impl IpAddr for V6 {
    fn display(&self) {
        println!("ipv6: {:?}",self.0)
    }
}

fn main() {
    let v: Vec<Box<dyn IpAddr>> = vec![
        Box::new(V4("127.0.0.1".to_string())),
        Box::new(V6("::1".to_string())),
    ];

    for ip in v {
        ip.display();
    }
}
```

在实际使用场景中，特征对象数组要比枚举数组常见很多，主要原因在于特征对象非常灵活，而编译器对枚举的限制较多，且无法动态增加类型。

### 13.7. Vector 常用方法

> 初始化 vec 的方式

```rust
fn main() {
    let v = vec![0; 3];   // 默认值为 0，初始长度为 3
    let v_from = Vec::from([0, 0, 0]);
    assert_eq!(v, v_from);
}
```

动态数组意味着我们增加元素时，如果容量不足就会导致 vector 扩容（目前的策略是重新申请一块 2 倍大小的内存，再将所有元素拷贝到新的内存位置，同时更新指针数据），显然，当频繁扩容或者当元素数量较多且需要扩容时，大量的内存拷贝会降低程序的性能。

可以考虑在初始化时就指定一个实际的预估容量，尽量减少可能的内存拷贝：

```rust
fn main() {
    let mut v = Vec::with_capacity(10);
    v.extend([1, 2, 3]);    // 附加数据到 v
    println!("Vector 长度是: {}, 容量是: {}", v.len(), v.capacity());

    v.reserve(100);        // 调整 v 的容量，至少要有 100 的容量
    println!("Vector（reserve） 长度是: {}, 容量是: {}", v.len(), v.capacity());

    v.shrink_to_fit();     // 释放剩余的容量，一般情况下，不会主动去释放容量
    println!("Vector（shrink_to_fit） 长度是: {}, 容量是: {}", v.len(), v.capacity());
}
```

### 13.8. Vector 的排序

两种排序算法，分别为稳定的排序 sort 和 sort_by，以及非稳定排序 sort_unstable 和 sort_unstable_by。

 非稳定 并不是指排序算法本身不稳定，而是指在排序过程中对相等元素的处理方式。在 稳定 排序算法里，对相等的元素，不会对其进行重新排序。而在 不稳定 的算法里则不保证这点。

总体而言，非稳定 排序的算法的速度会优于 稳定 排序算法，同时，稳定 排序还会额外分配原数组一半的空间。

> 整数数组的排序

以下是对整数列进行排序的例子。

```rust
fn main() {
    let mut vec = vec![1, 5, 10, 2, 15];    
    vec.sort_unstable();    
    assert_eq!(vec, vec![1, 2, 5, 10, 15]);
}
```

> 浮点数数组的排序

原来，在浮点数当中，存在一个 NAN 的值，这个值无法与其他的浮点数进行对比，因此，浮点数类型并没有实现全数值可比较 Ord 的特性，而是实现了部分可比较的特性 PartialOrd。

如此，如果我们确定在我们的浮点数数组当中，不包含 NAN 值，那么我们可以使用 partial_cmp 来作为大小判断的依据。

```rust
fn main() {
    let mut vec = vec![1.0, 5.6, 10.3, 2.0, 15f32];    
    vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());    
    assert_eq!(vec, vec![1.0, 2.0, 5.6, 10.3, 15f32]);
}
```

> 对结构体数组进行排序

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: String, age: u32) -> Person {
        Person { name, age }
    }
}

fn main() {
    let mut people = vec![
        Person::new("Zoe".to_string(), 25),
        Person::new("Al".to_string(), 60),
        Person::new("John".to_string(), 1),
    ];
    // 定义一个按照年龄倒序排序的对比函数
    people.sort_unstable_by(|a, b| b.age.cmp(&a.age));

    println!("{:?}", people);
}
```

## 14. HashMap

HashMap 也是 Rust 标准库中提供的集合类型，但是又与动态数组不同，HashMap 中存储的是一一映射的 KV 键值对，并提供了平均复杂度为 O(1) 的查询方法，当我们希望通过一个 Key 去查询值时，该类型非常有用。

Rust 中哈希类型（哈希映射）为 HashMap<K,V>，在其它语言中，也有类似的数据结构，例如 hash map，map，object，hash table，字典 等等。

### 14.1. 创建 HashMap

跟创建动态数组 Vec 的方法类似，可以使用 new 方法来创建 HashMap，然后通过 insert 方法插入键值对。

> 使用 new 方法创建

```rust
use std::collections::HashMap;

// 创建一个HashMap，用于存储宝石种类和对应的数量
let mut my_gems = HashMap::new();

// 将宝石类型和对应的数量写入表中
my_gems.insert("红宝石", 1);
my_gems.insert("蓝宝石", 2);
my_gems.insert("河边捡的误以为是宝石的破石头", 18);
```

使用 HashMap 需要手动通过 use ... 从标准库中引入到我们当前的作用域中来，之前使用另外两个集合类型 String 和 Vec 时，我们是否有手动引用过？答案是 No，因为 HashMap 并没有包含在 Rust 的 prelude 中（Rust 为了简化用户使用，提前将最常用的类型自动引入到作用域中）。

所有的集合类型都是动态的，意味着它们没有固定的内存大小，因此它们底层的数据都存储在内存堆上，然后通过一个存储在栈中的引用类型来访问。同时，跟其它集合类型一致，HashMap 也是内聚性的，即所有的 K 必须拥有同样的类型，V 也是如此。

跟 Vec 一样，如果预先知道要存储的 KV 对个数，可以使用 HashMap::with_capacity(capacity) 创建指定大小的 HashMap，避免频繁的内存分配和拷贝，提升性能。

> 使用迭代器和 collect 方法创建

into_iter 方法将列表转为迭代器，接着通过 collect 进行收集，不过需要注意的是，collect 方法在内部实际上支持生成多种类型的目标集合，因此我们需要通过类型标注 HashMap<_,_> 来告诉编译器：请帮我们收集为 HashMap 集合类型

```rust
fn main() {
    use std::collections::HashMap;

    let teams_list = vec![
        ("中国队".to_string(), 100),
        ("美国队".to_string(), 10),
        ("日本队".to_string(), 50),
    ];

    let teams_map: HashMap<_,_> = teams_list.into_iter().collect();
    
    println!("{:?}",teams_map)
}
```

### 14.2. 所有权转移

- 若类型实现 Copy 特征，该类型会被复制进 HashMap，因此无所谓所有权
- 若没实现 Copy 特征，所有权将被转移给 HashMap 中

```rust
fn main() {
    use std::collections::HashMap;

    let name = String::from("Sunface");
    let age = 18;

    let mut handsome_boys = HashMap::new();
    handsome_boys.insert(name, age);

    println!("因为过于无耻，{}已经被从帅气男孩名单中除名", name);
    println!("还有，他的真实年龄远远不止{}岁", age);
}
```

运行代码，报错如下：

```rust
error[E0382]: borrow of moved value: `name`
  --> src/main.rs:10:32
   |
4  |     let name = String::from("Sunface");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
...
8  |     handsome_boys.insert(name, age);
   |                          ---- value moved here
9  |
10 |     println!("因为过于无耻，{}已经被除名", name);
   |                                            ^^^^ value borrowed here after move
```

提示很清晰，name 是 String 类型，因此它受到所有权的限制，在 insert 时，它的所有权被转移给 handsome_boys，所以最后在使用时，会遇到这个无情但是意料之中的报错。

### 14.3. 查询 HashMap

通过 get 方法可以获取元素：

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();

scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);

let team_name = String::from("Blue");
let score: Option<&i32> = scores.get(&team_name);
```

- get 方法返回一个 Option<&i32> 类型：当查询不到时，会返回一个 None，查询到时返回 Some(&i32)
- &i32 是对 HashMap 中值的借用，如果不使用借用，可能会发生所有权的转移

如果我们想直接获得值类型的 score 该怎么办，答案简约但不简单:

```rust
let score: i32 = scores.get(&team_name).copied().unwrap_or(0);
```

### 14.4. 更新 HashMap 中的值

```rust
fn main() {
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert("Blue", 10);

    // 覆盖已有的值
    let old = scores.insert("Blue", 20);
    assert_eq!(old, Some(10));

    // 查询新插入的值
    let new = scores.get("Blue");
    assert_eq!(new, Some(&20));

    // 查询Yellow对应的值，若不存在则插入新值
    let v = scores.entry("Yellow").or_insert(5);
    assert_eq!(*v, 5); // 不存在，插入5

    // 查询Yellow对应的值，若不存在则插入新值
    let v = scores.entry("Yellow").or_insert(50);
    assert_eq!(*v, 5); // 已经存在，因此50没有插入
}
```

常用场景如下：查询某个 key 对应的值，若不存在则插入新值，若存在则对已有的值进行更新，例如在文本中统计词语出现的次数：

```rust
use std::collections::HashMap;

let text = "hello world wonderful world";

let mut map = HashMap::new();
// 根据空格来切分字符串(英文单词都是通过空格切分)
for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0);
    *count += 1;
}

println!("{:?}", map);
```

上面代码中，新建一个 map 用于保存词语出现的次数，插入一个词语时会进行判断：若之前没有插入过，则使用该词语作 Key，插入次数 0 作为 Value，若之前插入过则取出之前统计的该词语出现的次数，对其加一。

有两点值得注意：

- or_insert 返回了 &mut v 引用，因此可以通过该可变引用直接修改 map 中对应的值
- 使用 count 引用时，需要先进行解引用 *count，否则会出现类型不匹配

### 14.5. 哈希函数

```rust
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
// 引入第三方的哈希函数
use twox_hash::XxHash64;

// 指定HashMap使用第三方的哈希函数XxHash64
let mut hash: HashMap<_, _, BuildHasherDefault<XxHash64>> = Default::default();
hash.insert(42, "the answer");
assert_eq!(hash.get(&42), Some(&"the answer"));
```

目前，HashMap 使用的哈希函数是 SipHash，它的性能不是很高，但是安全性很高。SipHash 在中等大小的 Key 上，性能相当不错，但是对于小型的 Key （例如整数）或者大型 Key （例如字符串）来说，性能还是不够好。

## 15. 生命周期

### 15.1. 悬垂指针和生命周期

生命周期的主要作用是避免悬垂引用，它会导致程序引用了本不该引用的数据：

```rust
{
    let r;

    {
        let x = 5;
        r = &x;
    }

    println!("r: {}", r);
}
```

有几点值得注意:

- let r; 的声明方式貌似存在使用 null 的风险，实际上，当我们不初始化它就使用时，编译器会给予报错
- r 引用了内部花括号中的 x 变量，但是 x 会在内部花括号 } 处被释放，因此回到外部花括号后，r 会引用一个无效的 x  

此处 r 就是一个悬垂指针，它引用了提前被释放的变量 x，可以预料到，这段代码会报错：

```rust
error[E0597]: `x` does not live long enough // `x` 活得不够久
  --> src/main.rs:
   |
   |             r = &x;
   |                 ^^ borrowed value does not live long enough // 被借用的 `x` 活得不够久
   |         }
   |         - `x` dropped here while still borrowed // `x` 在这里被丢弃，但是它依然还在被借用
   |
   |         println!("r: {}", r);
   |                           - borrow later used here // 对 `x` 的借用在此处被使用
```

在这里 r 拥有更大的作用域，或者说活得更久。如果 Rust 不阻止该悬垂引用的发生，那么当 x 被释放后，r 所引用的值就不再是合法的，会导致我们程序发生异常行为，且该异常行为有时候会很难被发现。

### 15.2. 借用检查


### 15.3. 函数中的生命周期

生命周期的语法也颇为与众不同，以 ' 开头，名称往往是一个单独的小写字母，大多数人都用 'a 来作为生命周期的名称。 如果是引用类型的参数，那么生命周期会位于引用符号 & 之后，并用一个空格来将生命周期和引用参数分隔开:

```text
&i32        // 一个引用
&'a i32     // 具有显式生命周期的引用
&'a mut i32 // 具有显式生命周期的可变引用
```

一个生命周期标注，它自身并不具有什么意义，因为生命周期的作用就是告诉编译器多个引用之间的关系。例如，有一个函数，它的第一个参数 first 是一个指向 i32 类型的引用，具有生命周期 'a，该函数还有另一个参数 second，它也是指向 i32 类型的引用，并且同样具有生命周期 'a。此处生命周期标注仅仅说明，这两个参数 first 和 second 至少活得和'a 一样久，至于到底活多久或者哪个活得更久, 无法得知：

## 16. 方法 Method

### 16.1. 定义方法

Rust 使用 impl 来定义方法，例如以下代码：

```rust
struct Rectangle {
  width: u32,
  height: u32,
}

impl Rectangle {
  fn area(&self) -> u32 {
    self.width * self.height
  }
}

fn main() {
  let rect1 = Rectangle {
    width: 30,
    height: 50,
  };

  println!("{:?}", rect1.area())
}
```

Rust 的对象定义和方法定义是分离的，struct Rectangle 和 impl Rectangle，这种数据和使用分离的方式，会给予使用者极高的灵活度。  

impl Rectangle {} 表示为 Rectangle 实现方法(impl 是实现 implementation 的缩写)，这样的写法表明 impl 语句块中的一切都是跟 Rectangle 相关联的。  

### 16.2. self、&self 和 &mut self

在 area 的签名中，我们使用 &self 替代 rectangle: &Rectangle，&self 其实是 self: &Self 的简写（注意大小写）。在一个 impl 块内，Self 指代被实现方法的结构体类型，self 指代此类型的实例, self 指代的是 Rectangle 结构体实例.  

为哪个结构体实现方法，那么 self 就是指代哪个结构体的实例。  

self 依然有所有权的概念：

- self 表示 Rectangle 的所有权转移到该方法中，这种形式用的较少
- &self 表示该方法对 Rectangle 的不可变借用
- &mut self 表示可变借用

选择 &self 的理由跟在函数中使用 &Rectangle 是相同的：我们并不想获取所有权，也无需去改变它，只是希望能够读取结构体中的数据。如果想要在方法中去改变当前的结构体，需要将第一个参数改为 &mut self。仅仅通过使用 self 作为第一个参数来使方法获取实例的所有权是很少见的，这种使用方式往往用于把当前的对象转成另外一个对象时使用，转换完后，就不再关注之前的对象，且可以防止对之前对象的误调用。

### 16.3. 方法名跟结构体字段名相同

在 Rust 中，允许方法名跟结构体的字段名相同：

```rust
impl Rectangle {
    fn width(&self) -> bool {
        self.width > 0
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    if rect1.width() {
        println!("The rectangle has a nonzero width; it is {}", rect1.width);
    }
}
```

当我们使用 rect1.width() 时，Rust 知道我们调用的是它的方法，如果使用 rect1.width，则是访问它的字段。

一般来说，方法跟字段同名，往往适用于实现 getter 访问器，例如:

```rust
pub struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    pub fn new(width: u32, height: u32) -> Self {
        Rectangle { width, height }
    }
    pub fn width(&self) -> u32 {
        return self.width;
    }
}

fn main() {
    let rect1 = Rectangle::new(30, 50);

    println!("{}", rect1.width());
}
```

用这种方式，可以把 Rectangle 的字段设置为私有属性，只需把它的 new 和 width 方法设置为公开可见，那么用户就可以创建一个矩形，同时通过访问器 rect1.width() 方法来获取矩形的宽度，因为 width 字段是私有的，当用户访问 rect1.width 字段时，就会报错。注意在此例中，Self 指代的就是被实现方法的结构体 Rectangle。  

### 16.4. 带有多个参数的方法

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    let rect2 = Rectangle { width: 10, height: 40 };
    let rect3 = Rectangle { width: 60, height: 45 };

    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
}
```

### 16.5. 关联函数

在 impl 中且没有 self 的函数被称之为关联函数： 因为它没有 self，不能用 f.read() 的形式调用，因此它是一个函数而不是方法，它又在 impl 中，与结构体紧密关联，因此称为关联函数。

```rust
impl Rectangle {
    fn new(w: u32, h: u32) -> Rectangle {
        Rectangle { width: w, height: h }
    }
}
```

因为是函数，所以不能用 . 的方式来调用，我们需要用 :: 来调用，例如 let sq = Rectangle::new(3, 3);。这个方法位于结构体的命名空间中：:: 语法用于关联函数和模块创建的命名空间。

### 16.6. 多个 impl 定义

Rust 允许我们为一个结构体定义多个 impl 块，目的是提供更多的灵活性和代码组织性，例如当方法多了后，可以把相关的方法组织在同一个 impl 块中，那么就可以形成多个 impl 块，各自完成一块儿目标：

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```

### 16.7. 为枚举实现方法

枚举类型之所以强大，不仅仅在于它好用、可以同一化类型，还在于，我们可以像结构体一样，为枚举实现方法：

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        // 在这里定义方法体
    }
}

fn main() {
    let m = Message::Write(String::from("hello"));
    m.call();
}
```

## 17. 使用 macro_rules! 来创建宏

Rust 提供了一个强大的宏系统，可进行元编程（metaprogramming）。宏并不产生函数调用，而是展开成源码，并和程序的其余部分一起被编译。Rust 又有一点和 C 以及其他语言都不同，那就是 Rust 的宏会展开为抽象语法树（AST，abstract syntax tree），而不是像字符串预处理那样直接替换成代码，这样就不会产生无法预料的优先权错误。

宏是通过 macro_rules! 宏来创建的。

```rust
// 这是一个简单的宏，名为 `say_hello`。
macro_rules! say_hello {
    // `()` 表示此宏不接受任何参数。
    () => (
        // 此宏将会展开成这个代码块里面的内容。
        println!("Hello!");
    )
}

fn main() {
    // 这个调用将会展开成 `println("Hello");`!
    say_hello!()
}
```

为什么宏是有用的？

- 不写重复代码（DRY，Don't repeat yourself.）。很多时候你需要在一些地方针对不同 的类型实现类似的功能，可以使用宏来避免重复代码。
- 领域专用语言（DSL，domain-specific language）。宏允许你为特定的目的创造特定的语法。
- 可变接口（variadic interface）。能够接受不定数目参数的接口，比如 println!，根据格式化字符串的不同，它需要接受任意多的参数。

### 17.1. 指示符

宏的参数使用一个美元符号 $ 作为前缀，并使用一个指示符（designator）来注明类型(https://doc.rust-lang.org/reference/macros-by-example.html)：

- block 块表达式
- expr 用于表达式
- ident 用于变量名或函数名
- item Item
- literal 用于字面常量
- pat 模式 pattern
- path TypePath 样式路径
- stmt 语句 statement
- tt 标记树 token tree
- ty 类型 type
- vis 可见性描述符

```rust
macro_rules! print_result {
    // 此宏接受一个 `expr` 类型的表达式，并将它作为字符串，连同其结果一起打印出来。
    // `expr` 指示符表示表达式。
    ($expression:expr) => {
        // `stringify!` 把表达式*原样*转换成一个字符串。
        println!("{:?} = {:?}", stringify!($expression), $expression)
    };
}

fn main() {
    foo();
    bar();
    print_result!(1u32 + 1); // "1u32 + 1" = 2

    // 代码块也是表达式！
    print_result!({
        let x = 1u32;
        x * x + 2 * x - 1
    }); // "{ let x = 1u32; x * x + 2 * x - 1 }" = 2
}
```

### 17.2. 重载

宏可以重载，从而接受不同的参数组合。在这方面，macro_rules! 的作用类似于匹配（match）代码块：

```rust
macro_rules! test {
    // 参数不需要使用逗号隔开。 参数可以任意组合！
    ($left:expr; and $right:expr) => {
        println!(
            "{:?} and {:?} is {:?}",
            stringify!($left),
            stringify!($right),
            $left && $right
        )
    };
    // 每个分支都必须以分号结束。
    ($left:expr; or $right:expr) => {
        println!(
            "{:?} or {:?} is {:?}",
            stringify!($left),
            stringify!($right),
            $left || $right
        )
    };
}

fn main() {
    test!(1i32 + 1 == 2i32; and 2i32 * 2 == 4i32); // "1i32 + 1 == 2i32" and "2i32 * 2 == 4i32" is true
    test!(true; or false); // "true" or "false" is true
}
```

### 17.3. 重复

宏在参数列表中可以使用 + 来表示一个参数可能出现一次或多次，使用 * 来表示该参数可能出现零次或多次。

> $(...),+ 包围起来，就可以匹配一个或多个用逗号隔开的表达式。另外注意到，宏定义的最后一个分支可以不用分号作为结束。

```rust
macro_rules! find_min {
    // 基本情形：
    ($x:expr) => ($x);
    // `$x` 后面跟着至少一个 `$y,`
    ($x:expr, $($y:expr),+) => (
        // 对 `$x` 后面的 `$y` 们调用 `find_min!`
        std::cmp::min($x, find_min!($($y),+))
    )
}

fn main() {
    println!("{}", find_min!(1u32)); // 1
    println!("{}", find_min!(1u32 + 2, 2u32)); // 2
    println!("{}", find_min!(5u32, 2u32 * 3, 4u32)); // 4
}
```

### 17.4. DRY (不写重复代码)

通过提取函数或测试集的公共部分，宏可以让你写出 DRY 的代码, 一个例子，对 Vec<T> 实现并测试了关于 +=、*= 和 -= 等运算符。

```rust
use std::ops::{Add, Mul, Sub};

macro_rules! assert_equal_len {
    // `tt`（token tree，标记树）指示符表示运算符和标记。
    ($a:ident, $b: ident, $func:ident, $op:tt) => {
        assert!(
            $a.len() == $b.len(),
            "{:?}: dimension mismatch: {:?} {:?} {:?}",
            stringify!($func),
            ($a.len(),),
            stringify!($op),
            ($b.len(),)
        );
    };
}

macro_rules! op {
    ($func:ident, $bound:ident, $op:tt, $method:ident) => {
        fn $func<T: $bound<T, Output = T> + Copy>(xs: &mut Vec<T>, ys: &Vec<T>) {
            assert_equal_len!(xs, ys, $func, $op);

            for (x, y) in xs.iter_mut().zip(ys.iter()) {
                *x = $bound::$method(*x, *y);
                // *x = x.$method(*y); // 效果同上
            }
        }
    };
}

// 实现 `add_assign`、`mul_assign` 和 `sub_assign` 等函数。
op!(add_assign, Add, +=, add);
op!(mul_assign, Mul, *=, mul);
op!(sub_assign, Sub, -=, sub);
```

### 17.5. DSL（领域专用语言）

DSL 是 Rust 的宏中集成的微型 “语言”。这种语言是完全合法的，因为宏系统会把它转换成普通的 Rust 语法树，它只不过看起来像是另一种语言而已。这就允许你为一些特定功能创造一套简洁直观的语法（当然是有限制的）。

定义一套小的计算器 API，可以传给它表达式，它会把结果打印到控制台上。

```rust
macro_rules! calculate {
    (eval $e:expr) => {{
        {
            let val: usize = $e; // 强制类型为整型
            println!("{} = {}", stringify!{$e}, val);
        }
    }};
}

fn main() {
    calculate! {
        eval 1 + 2 // `eval` 可并不是 Rust 的关键字！
    } // 1 + 2 = 3

    calculate! {
        eval (1 + 2) * (3 / 4)
    } // (1 + 2) * (3 / 4) = 0   ??
}
```

### 17.6. 可变参数接口

可变参数接口可以接受任意数目的参数。比如说 println 就可以，其参数的数目是由格式化字符串指定的。

```rust
macro_rules! calculate {
    // 单个 `eval` 的模式
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};

    // 递归地拆解多重的 `eval`
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

fn main() {
    calculate! { // 可变参数的 `calculate!`！
        eval 1 + 2,
        eval 3 + 4,
        eval (2 * 3) + 1,
        eval 6 + 4
    }
    // 1 + 2 = 3
    // 3 + 4 = 7
    // (2 * 3) + 1 = 7
    // 6 + 4 = 10
}
```

## 18. 错误处理

错误处理（error handling）是处理可能发生的失败情况的过程。例如读取一个文件时失败了，如果继续使用这个无效的输入，那显然是有问题的。注意到并且显式地处理这种错误可以避免程序的其他部分产生潜在的问题。

在 Rust 中有多种处理错误的方式:

- 显式的 panic 主要用于测试，以及处理不可恢复的错误。在原型开发中这很有用，比如 用来测试还没有实现的函数，不过这时使用 unimplemented 更能表达意图。另外在 测试中，panic 是一种显式地失败（fail）的好方法。
- Option 类型是为了值是可选的、或者缺少值并不是错误的情况准备的。比如说寻找 父目录时，/ 和 C: 这样的目录就没有父目录，这应当并不是一个错误。当处理 Option 时，unwrap 可用于原型开发，也可以用于能够确定 Option 中一定有值 的情形。然而 expect 更有用，因为它允许你指定一条错误信息，以免万一还是出现 了错误。
- 当错误有可能发生，且应当由调用者处理时，使用 Result。你也可以 unwrap 然后 使用 expect，但是除了在测试或者原型开发中，请不要这样做。

### 18.1. panic

最简单的错误处理机制就是 panic。它会打印一个错误消息，开始回退（unwind）任务，且通常会退出程序。这里我们显式地在错误条件下调用 panic：

```rust
fn give_princess(gift: &str) {
    if gift == "snake" {
        panic!("AAAaaaaa!!!!");
    }

    println!("I love {}s!!!!!", gift);
}

fn main() {
    give_princess("teddy bear");
    give_princess("snake");
}
```

### 18.2. Option 和 unwrap

在标准库（std）中有个叫做 Option<T>（option 中文意思是 “选项”）的枚举类型，用于有 “不存在” 的可能性的情况。它表现为以下两个 “option”（选项）中的一个：

- Some(T)：找到一个属于 T 类型的元素
- None：找不到相应元素

这些选项可以通过 match 显式地处理，或使用 unwrap 隐式地处理。隐式处理要么返回 Some 内部的元素，要么就 panic。

手动使用 expect 方法自定义 panic 信息是可能的，但相比显式处理，unwrap 的输出仍显得不太有意义。在下面例子中，显式处理将举出更受控制的结果，同时如果需要的话，仍然可以使程序 panic。

```rust
// 显式地使用 `match` 来处理。
fn give_commoner(gift: Option<&str>) {
    // 指出每种情况下的做法。
    match gift {
        Some("snake") => println!("Yuck! I'm throwing that snake in a fire."),
        Some(inner) => println!("{}? How nice.", inner),
        None => println!("No gift? Oh well."),
    }
}

// 使用 `unwrap` 隐式地处理。
fn give_princess(gift: Option<&str>) {
    // `unwrap` 在接收到 `None` 时将返回 `panic`。
    let inside = gift.unwrap();
    if inside == "snake" {
        panic!("AAAaaaaa!!!!");
    }

    println!("I love {}s!!!!!", inside);
}

fn main() {
    let food = Some("chicken");
    let snake = Some("snake");
    let void = None;

    give_commoner(food); // chicken? How nice.
    give_commoner(snake); // Yuck! I'm throwing that snake in a fire.
    give_commoner(void); // No gift? Oh well.

    let bird = Some("robin");
    let nothing = None;

    give_princess(bird); // I love robins!!!!!
    give_princess(nothing); // thread 'main' panicked at src/main.rs:29:23:   called `Option::unwrap()` on a `None` value
}
```

### 18.3. 使用 ? 解开 Option

如果 x 是 Option，那么若 x 是 Some ，对x?表达式求值将返回底层值，否则无论函数是否正在执行都将终止且返回 None。

```rust
struct Person {
    job: Option<Job>,
}

#[derive(Clone, Copy)]
struct Job {
    phone_number: Option<PhoneNumber>,
}

#[derive(Clone, Copy)]
struct PhoneNumber {
    area_code: Option<u8>,
    number: u32,
}

impl Person {
    // 获取此人的工作电话号码的区号（如果存在的话）。
    fn work_phone_area_code(&self) -> Option<u8> {
        // 没有`？`运算符的话，这将需要很多的嵌套的 `match` 语句。
        self.job?.phone_number?.area_code
    }
}

fn main() {
    let p = Person {
        job: Some(Job {
            phone_number: Some(PhoneNumber {
                area_code: Some(61),
                number: 439222222,
            }),
        }),
    };

    println!("{:?}", p.work_phone_area_code()); // Some(61)
}
```

### 18.4. 组合算子：map

match 是处理 Option 的一个可用的方法，但你会发现大量使用它会很繁琐，特别是当操作只对一种输入是有效的时。这时，可以使用组合算子（combinator），以模块化的风格来管理控制流。

Option 有一个内置方法 map()，这个组合算子可用于 Some -> Some 和 None -> None 这样的简单映射。多个不同的 map() 调用可以串起来，这样更加灵活。

在下面例子中，process() 轻松取代了前面的所有函数，且更加紧凑。

```rust
#[derive(Debug)]
enum Food {
    Apple,
    Carrot,
    Potato,
}

#[derive(Debug)]
struct Peeled(Food);
#[derive(Debug)]
struct Chopped(Food);
#[derive(Debug)]
struct Cooked(Food);

// 削皮。如果没有食物，就返回 `None`。否则返回削好皮的食物。
fn peel(food: Option<Food>) -> Option<Peeled> {
    match food {
        Some(food) => Some(Peeled(food)),
        None => None,
    }
}

// 切食物。如果没有食物，就返回 `None`。否则返回切好的食物。
fn chop(peeled: Option<Peeled>) -> Option<Chopped> {
    match peeled {
        Some(Peeled(food)) => Some(Chopped(food)),
        None => None,
    }
}

// 烹饪食物。这里，我们使用 `map()` 来替代 `match` 以处理各种情况。
fn cook(chopped: Option<Chopped>) -> Option<Cooked> {
    chopped.map(|Chopped(food)| Cooked(food))
}

// 这个函数会完成削皮切块烹饪一条龙。我们把 `map()` 串起来，以简化代码。
fn process(food: Option<Food>) -> Option<Cooked> {
    food.map(|f| Peeled(f))
        .map(|Peeled(f)| Chopped(f))
        .map(|Chopped(f)| Cooked(f))
}

// 在尝试吃食物之前确认食物是否存在是非常重要的！
fn eat(food: Option<Cooked>) {
    match food {
        Some(food) => println!("Mmm. I love {:?}", food),
        None => println!("Oh no! It wasn't edible."),
    }
}

fn main() {
    let apple = Some(Food::Apple);
    let carrot = Some(Food::Carrot);
    let potato = None;

    let cooked_apple = cook(chop(peel(apple)));
    // let cooked_carrot = cook(chop(peel(carrot)));
    let cooked_carrot = process(carrot); // 结果同上

    // 看起来更简单的 `process()`
    let cooked_potato = process(potato);

    eat(cooked_apple); // Mmm. I love Cooked(Apple)
    eat(cooked_carrot); // Mmm. I love Cooked(carrot)
    eat(cooked_potato); // Oh no! It wasn't edible.
}
```

### 18.5. 组合算子：and_then

map() 以链式调用的方式来简化 match 语句。然而，如果以返回类型是 Option<T> 的函数作为 map() 的参数，会导致出现嵌套形式 Option<Option<T>>。这样多层串联调用就会变得混乱。所以有必要引入 and_then()，在某些语言中它叫做 flatmap。

and_then() 使用被 Option 包裹的值来调用其输入函数并返回结果。 如果 Option 是 None，那么它返回 None。

```rust
enum Food {
    CordonBleu,
    Steak,
    Sushi,
}
#[derive(Debug)]
enum Day {
    Monday,
    Tuesday,
    Wednesday,
}

// 没有制作寿司所需的原材料（ingredient）（有其他的原材料）。
fn have_ingredients(food: Food) -> Option<Food> {
    match food {
        Food::Sushi => None,
        _ => Some(food),
    }
}

// 拥有全部食物的食谱，除了法国蓝带猪排（Cordon Bleu）的。
fn have_recipe(food: Food) -> Option<Food> {
    match food {
        Food::CordonBleu => None,
        _ => Some(food),
    }
}

// 一系列 `match` 来表达这个逻辑：
fn cookable_v1(food: Food) -> Option<Food> {
    match have_ingredients(food) {
        None => None,
        Some(food) => match have_recipe(food) {
            None => None,
            Some(food) => Some(food),
        },
    }
}

// 也可以使用 `and_then()` 把上面的逻辑改写得更紧凑：
fn cookable_v2(food: Food) -> Option<Food> {
    have_ingredients(food).and_then(have_recipe)
}

fn eat(food: Food, day: Day) {
    match cookable_v2(food) {
        Some(food) => println!("Yay! On {:?} we get to eat {:?}.", day, food),
        None => println!("Oh no. We don't get to eat on {:?}?", day),
    }
}

fn main() {
    let (cordon_bleu, steak, sushi) = (Food::CordonBleu, Food::Steak, Food::Sushi);

    eat(cordon_bleu, Day::Monday); // Oh no. We don't get to eat on Monday?
    eat(steak, Day::Tuesday); // Yay! On Tuesday we get to eat Steak.
    eat(sushi, Day::Wednesday); // Oh no. We don't get to eat on Wednesday?
}
```

### 18.6. 结果 Result

Result 是 Option 类型的更丰富的版本，描述的是可能的错误而不是可能的不存在。

也就是说，Result<T，E> 可以有两个结果的其中一个：

- Ok<T>：找到 T 元素
- Err<E>：找到 E 元素，E 即表示错误的类型。

按照约定，预期结果是 “Ok”，而意外结果是 “Err”。

Result 有很多类似 Option 的方法。例如 unwrap()，它要么举出元素 T，要么就 panic。 对于事件的处理，Result 和 Option 有很多相同的组合算子。

在使用 Rust 时，你可能会遇到返回 Result 类型的方法，例如 parse() 方法。它并不是总能把字符串解析成指定的类型，所以 parse() 返回一个 Result 表示可能的失败。

```rust
fn multiply(first_number_str: &str, second_number_str: &str) -> i32 {
    let first_number = first_number_str.parse::<i32>().unwrap();
    let second_number = second_number_str.parse::<i32>().unwrap();
    first_number * second_number
}

fn main() {
    let twenty = multiply("10", "2");
    println!("double is {}", twenty); // double is 20

    let tt = multiply("t", "2");
    println!("double is {}", tt); // called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
}
```

在失败的情况下，parse() 产生一个错误，留给 unwrap() 来解包并产生 panic。另外，panic 会退出我们的程序，并提供一个让人很不爽的错误消息。

为了改善错误消息的质量，我们应该更具体地了解返回类型并考虑显式地处理错误。

### 18.7. Result 的 map

一般地，希望把错误返回给调用者，这样它可以决定回应错误的正确方式。

首先，需要了解需要处理的错误类型是什么。为了确定 Err 的类型，可以用 parse() 来试验。Rust 已经为 i32 类型使用 FromStr trait 实现了 parse()。结果表明，这里的 Err 类型被指定为 ParseIntError。

```rust
fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
    first_number_str.parse::<i32>().and_then(|first_number| {
        second_number_str
            .parse::<i32>()
            .map(|second_number| first_number * second_number)
    })
}

fn print(result: Result<i32, ParseIntError>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    // 这种情况下仍然会给出正确的答案。
    let twenty = multiply("10", "2");
    print(twenty); // n is 20

    // 这种情况下就会提供一条更有用的错误信息。
    let tt = multiply("t", "2");
    print(tt); // Error: invalid digit found in string
}
```

### 18.8. 给 Result 取别名

在模块的层面上创建别名特别有帮助。同一模块中的错误常常会有相同的 Err 类型，所以单个别名就能简便地定义所有相关的 Result。标准库也提供了一个别名： io::Result！

```rust
use std::num::ParseIntError;

// 为带有错误类型 `ParseIntError` 的 `Result` 定义一个泛型别名。
type AliasedResult<T> = Result<T, ParseIntError>;

// 使用上面定义过的别名来表示上一节中的 `Result<i32,ParseIntError>` 类型。
fn multiply(first_number_str: &str, second_number_str: &str) -> AliasedResult<i32> {
    first_number_str.parse::<i32>().and_then(|first_number| {
        second_number_str
            .parse::<i32>()
            .map(|second_number| first_number * second_number)
    })
}

// 在这里使用别名又让我们节省了一些代码量。
fn print(result: AliasedResult<i32>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    print(multiply("10", "2")); // n is 20
    print(multiply("t", "2")); // Error: invalid digit found in string
}
```

### 18.9. 提前返回

另一种处理错误的方式是使用 match 语句和提前返回（early return）的结合。

如果发生错误，我们可以停止函数的执行然后返回错误。对有些人来说，这样的代码更好写，更易读。这次我们使用提前返回改写之前的例子：

```rust
use std::num::ParseIntError;

fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
    let first_number = match first_number_str.parse::<i32>() {
        Ok(first_number) => first_number,
        Err(e) => return Err(e),
    };

    let second_number = match second_number_str.parse::<i32>() {
        Ok(second_number) => second_number,
        Err(e) => return Err(e),
    };

    Ok(first_number * second_number)
}

fn print(result: Result<i32, ParseIntError>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    print(multiply("10", "2")); // n is 20
    print(multiply("t", "2")); // Error: invalid digit found in string
}
```

### 18.10. 引入 ?

有时我们只是想 unwrap 且避免产生 panic。到现在为止，对 unwrap 的错误处理都在强迫我们一层层地嵌套，然而我们只是想把里面的变量拿出来。? 正是为这种情况准备的, ? 几乎就等于一个会返回 Err 而不是 panic 的 unwrap。

当找到一个 Err 时，可以采取两种行动：

- panic!，不过我们已经决定要尽可能避免 panic 了。
- 返回它，因为 Err 就意味着它已经不能被处理了。

```rust
use std::num::ParseIntError;

fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
    let first_number = first_number_str.parse::<i32>()?;
    let second_number = second_number_str.parse::<i32>()?;

    Ok(first_number * second_number)
}

fn print(result: Result<i32, ParseIntError>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    print(multiply("10", "2")); // n is 20
    print(multiply("t", "2")); // Error: invalid digit found in string
}
```

### 18.11. try! 宏

在 ? 出现以前，相同的功能是使用 try! 宏完成的。现在我们推荐使用 ? 运算符，但是在老代码中仍然会看到 try!。如果使用 try! 的话，上一个例子中的 multiply 函数看起来会像是这样：

```rust
use std::num::ParseIntError;

fn multiply(first_number_str: &str, second_number_str: &str) -> Result<i32, ParseIntError> {
    let first_number = r#try!(first_number_str.parse::<i32>());
    let second_number = r#try!(second_number_str.parse::<i32>());

    Ok(first_number * second_number)
}

fn print(result: Result<i32, ParseIntError>) {
    match result {
        Ok(n)  => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    print(multiply("10", "2")); // n is 20
    print(multiply("t", "2")); // Error: invalid digit found in string
}
```

### 18.12. 处理多种错误类型

有时 Option 需要和 Result 进行交互，或是 Result<T, Error1> 需要和 Result<T, Error2> 进行交互。在这类情况下，我们想要以一种方式来管理不同的错误类型，使得它们可组合且易于交互。

在下面代码中，unwrap 的两个实例生成了不同的错误类型。Vec::first 返回一个 Option，而 parse::<i32> 返回一个 Result<i32, ParseIntError>：

```rust
fn double_first(vec: Vec<&str>) -> i32 {
    let first = vec.first().unwrap(); // 生成错误 1
    2 * first.parse::<i32>().unwrap() // 生成错误 2
}

fn main() {
    let numbers = vec!["42", "93", "18"];
    let empty = vec![];
    let strings = vec!["tofu", "93", "18"];

    println!("The first doubled is {}", double_first(numbers)); // The first doubled is 84
 
    println!("The first doubled is {}", double_first(empty)); // called `Option::unwrap()` on a `None` value
    // 错误1：输入 vector 为空

    println!("The first doubled is {}", double_first(strings)); // called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
    // 错误2：此元素不能解析成数字
}
```

### 18.13. 从 Option 中取出 Result

处理混合错误类型的最基本的手段就是让它们互相包含。 Option 是 None 则继续处理错误。一些组合算子可以让我们轻松地交换 Result 和 Option。

```rust
use std::num::ParseIntError;

fn double_first(vec: Vec<&str>) -> Result<Option<i32>, ParseIntError> {
    let opt = vec.first().map(|first| first.parse::<i32>().map(|n| 2 * n));

    opt.map_or(Ok(None), |r| r.map(Some))
}

fn main() {
    let numbers = vec!["42", "93", "18"];
    let empty = vec![];
    let strings = vec!["tofu", "93", "18"];

    println!("The first doubled is {:?}", double_first(numbers)); // The first doubled is Ok(Some(84))
    println!("The first doubled is {:?}", double_first(empty)); // The first doubled is Ok(None)
    println!("The first doubled is {:?}", double_first(strings)); // The first doubled is Err(ParseIntError { kind: InvalidDigit })
}
```

### 18.14. 定义一个错误类型

Rust 允许我们定义自己的错误类型。一般来说，一个 “好的” 错误类型应当：

- 用同一个类型代表了多种错误
- 向用户提供了清楚的错误信息
- 能够容易地与其他类型比较
  - 好的例子：Err(EmptyVec)
  - 坏的例子：Err("Please use a vector with at least one element".to_owned())
- 能够容纳错误的具体信息
  - 好的例子：Err(BadChar(c, position))
  - 坏的例子：Err("+ cannot be used here".to_owned())
- 能够与其他错误很好地整合

```rust
use std::error;
use std::fmt;

type Result<T> = std::result::Result<T, DoubleError>;

#[derive(Debug, Clone)]
// 定义我们的错误类型，这种类型可以根据错误处理的实际情况定制。
// 可以完全自定义错误类型，也可以在类型中完全采用底层的错误实现，也可以介于二者之间。
struct DoubleError;

// 没有储存关于错误的任何额外信息，也就是说，如果不修改我们的错误类型定义的话，就无法指明是哪个字符串解析失败了。
impl fmt::Display for DoubleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// 为 `DoubleError` 实现 `Error` trait，这样其他错误可以包裹这个错误类型。
impl error::Error for DoubleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None // 泛型错误，没有记录其内部原因。
    }
}

fn double_first(vec: Vec<&str>) -> Result<i32> {
    vec.first()
        .ok_or(DoubleError) // 把错误换成我们的新类型。
        .and_then(|s| {
            s.parse::<i32>()
                .map_err(|_| DoubleError) // 这里也换成新类型。
                .map(|i| 2 * i)
        })
}

fn print(result: Result<i32>) {
    match result {
        Ok(n) => println!("The first doubled is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    let numbers = vec!["42", "93", "18"];
    let empty = vec![];
    let strings = vec!["tofu", "93", "18"];

    print(double_first(numbers)); // The first doubled is 84
    print(double_first(empty)); // Error: invalid first item to double
    print(double_first(strings)); // Error: invalid first item to double
}
```

### 18.15. 把错误 “装箱”

想写简单的代码，又想保存原始错误信息，一个方法是把它们装箱（Box）。这样做的坏处就是，被包装的错误类型只能在运行时了解，而不能被静态地判别。

### 18.16. ? 的其他用法

? 实际上是指 unwrap 或 return Err(From::from(err))。由于 From::from 是不同类型之间的转换工具，也就是说，如果在错误可转换成返回类型地方使用 ?，它将自动转换成返回类型。

```rust
// 使用 `?` 立即得到内部值。
fn double_first(vec: Vec<&str>) -> Result<i32> {
    let first = vec.first().ok_or(EmptyVec)?;
    let parsed = first.parse::<i32>()?;
    Ok(2 * parsed)
}
```

### 18.17. 包裹错误


### 18.18. 遍历Result

Iter::map 操作可能失败


#### 18.18.1. 使用 filter_map() 忽略失败的项

filter_map 会调用一个函数，过滤掉为 None 的所有结果。

```rust
fn main() {
    let strings = vec!["tofu", "93", "18"];
    let numbers: Vec<_> = strings
        .into_iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    println!("Results: {:?}", numbers); // Results: [93, 18]
}
```

#### 18.18.2. 使用 collect() 使整个操作失败

Result 实现了 FromIter，因此结果的向量（Vec<Result<T, E>>）可以被转换成结果包裹着向量（Result<Vec<T>, E>）。一旦找到一个 Result::Err ，遍历就被终止。

```rust
fn main() {
    let strings = vec!["tofu", "93", "18"];
    let numbers: Result<Vec<_>, _> = strings
        .into_iter()
        .map(|s| s.parse::<i32>())
        .collect();
    println!("Results: {:?}", numbers); // // Results: Err(ParseIntError { kind: InvalidDigit })
}
```

#### 18.18.3. 使用 Partition() 收集所有合法的值与错误

```rust
fn main() {
    let strings = vec!["tofu", "93", "18"];
    let (numbers, errors): (Vec<_>, Vec<_>) = strings
        .into_iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);
    println!("Numbers: {:?}", numbers); // Numbers: [Ok(93), Ok(18)]
    println!("Errors: {:?}", errors); // Errors: [Err(ParseIntError { kind: InvalidDigit })]
}
```












## x. 模块管理

Rust 有许多功能可以让你管理代码的组织，包括哪些内容可以被公开，哪些内容作为私有部分，以及程序每个作用域中的名字。这些功能被称为 “模块系统（the module system）”，包括：

- 包（Packages）： Cargo 的一个功能，它允许你构建、测试和分享 crate。
- Crates ：一个模块的树形结构，它形成了库或二进制项目。
- 模块（Modules）和 use： 允许你控制作用域和路径的私有性。
- 路径（path）：一个命名例如结构体、函数或模块等项的方式

### x.1. 包和 Crate

crate 是一个二进制项或者库。crate root 是一个源文件，Rust 编译器以它为起始点，并构成你的 crate 的根模块.  
包（package） 是提供一系列功能的一个或者多个 crate。一个包会包含有一个 Cargo.toml 文件，阐述如何去构建这些 crate。  
包中可以包含至多一个库 crate(library crate)。可以包含任意多个二进制 crate(binary crate)，但是必须至少包含一个 crate（无论是库的还是二进制的）。  

cargo new命令

```text
$ cargo new my-project
     Created binary (application) `my-project` package
$ ls my-project
Cargo.toml
src
$ ls my-project/src
main.rs
```

### x.2 定义模块来控制作用域与私有性

一个包含了其他内置了函数的模块的 front_of_house 模块

```rs
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}

        fn seat_at_table() {}
    }

    mod serving {
        fn take_order() {}

        fn serve_order() {}

        fn take_payment() {}
    }
}
```

模块树的结构。

```text
crate
 └── front_of_house
     ├── hosting
     │   ├── add_to_waitlist
     │   └── seat_at_table
     └── serving
         ├── take_order
         ├── serve_order
         └── take_payment
```

路径有两种形式：

- 绝对路径（absolute path）从 crate 根开始，以 crate 名或者字面值 crate 开头。
- 相对路径（relative path）从当前模块开始，以 self、super 或当前模块的标识符开头。

以下代码会编译失败，暂时忽略

```rs
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // 绝对路径
    crate::front_of_house::hosting::add_to_waitlist();

    // 相对路径
    front_of_house::hosting::add_to_waitlist();
}
```

> 使用 pub 关键字暴露路径

```rs
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // 绝对路径
    crate::front_of_house::hosting::add_to_waitlist();

    // 相对路径
    front_of_house::hosting::add_to_waitlist();
}
```

> 使用 super 起始的相对路径

使用 super 开头来构建从父模块开始的相对路径。这么做类似于文件系统中以 .. 开头的语法。

```rs
fn serve_order() {}

mod back_of_house {
    fn fix_incorrect_order() {
        cook_order();
        super::serve_order();
    }

    fn cook_order() {}
}
```

> 使用 use 关键字将路径引入作用域

在作用域中增加 use 和路径类似于在文件系统中创建软连接（符号连接，symbolic link）。通过在 crate 根增加 use crate::front_of_house::hosting，现在 hosting 在作用域中就是有效的名称了，如同 hosting 模块被定义于 crate 根一样。通过 use 引入作用域的路径也会检查私有性，同其它路径一样。

```rs
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}
```

> 使用 as 关键字提供新的名称

用 as 指定一个新的本地名称或者别名

```rs
use std::fmt::Result;
use std::io::Result as IoResult;

fn function1() -> Result {
    // --snip--
}

fn function2() -> IoResult<()> {
    // --snip--
}
```

### x.3 库 Package

```rust
$ cargo new my-lib --lib
     Created library `my-lib` package
$ ls my-lib
Cargo.toml
src
$ ls my-lib/src
lib.rs
```

#### x.3.1. Package 结构

一个真实项目中典型的 Package，会包含多个二进制包，这些包文件被放在 src/bin 目录下，每一个文件都是独立的二进制包，同时也会包含一个库包，该包只能存在一个 src/lib.rs：

```text
.
├── Cargo.toml
├── Cargo.lock
├── src
│   ├── main.rs
│   ├── lib.rs
│   └── bin
│       └── main1.rs
│       └── main2.rs
├── tests
│   └── some_integration_tests.rs
├── benches
│   └── simple_bench.rs
└── examples
    └── simple_example.rs
```

- 唯一库包：src/lib.rs
- 默认二进制包：src/main.rs，编译后生成的可执行文件与 Package 同名
- 其余二进制包：src/bin/main1.rs 和 src/bin/main2.rs，它们会分别生成一个文件同名的二进制可执行文件
- 集成测试文件：tests 目录下
- 基准性能测试 benchmark 文件：benches 目录下
- 项目示例：examples 目录下







