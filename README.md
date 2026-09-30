# rust

**English** | [简体中文](README.zh-CN.md)

Learning resources:  
The Rust Programming Language: https://doc.rust-lang.org/book/  
Chinese edition: https://kaisery.github.io/trpl-zh-cn/  

Standard library docs: https://doc.rust-lang.org/std/prelude/index.html  

- [rust](#rust)
  - [0. Repository layout](#0-repository-layout)
    - [0.1. Examples](#01-examples)
    - [0.2. Projects](#02-projects)
  - [1. Installing Rust (Mac)](#1-installing-rust-mac)
  - [2. Cargo](#2-cargo)
    - [2.1. Creating a project with Cargo](#21-creating-a-project-with-cargo)
      - [2.1.1. Dependencies from crates.io](#211-dependencies-from-cratesio)
      - [2.1.2. Dependencies from other registries](#212-dependencies-from-other-registries)
      - [2.1.3. Git repositories as dependencies](#213-git-repositories-as-dependencies)
      - [2.1.4. Local dependencies by path](#214-local-dependencies-by-path)
      - [2.1.5. Platform-specific dependencies](#215-platform-specific-dependencies)
      - [2.1.6. \[dev-dependencies\]](#216-dev-dependencies)
      - [2.1.7. \[build-dependencies\]](#217-build-dependencies)
      - [2.1.8 Choosing features](#218-choosing-features)
      - [2.1.9. Renaming dependencies in Cargo.toml](#219-renaming-dependencies-in-cargotoml)
    - [2.2. Building and running a Cargo project](#22-building-and-running-a-cargo-project)
    - [2.3. Cargo.toml format](#23-cargotoml-format)
    - [2.4. Release builds](#24-release-builds)
  - [3. Basic syntax](#3-basic-syntax)
    - [3.1. Variables](#31-variables)
    - [3.2. Constants](#32-constants)
    - [3.3. Shadowing](#33-shadowing)
  - [4. Data types](#4-data-types)
    - [4.1. Scalar types](#41-scalar-types)
      - [4.1.1. Integer types](#411-integer-types)
      - [4.1.2. Floating-point types](#412-floating-point-types)
    - [4.2. Numeric operations](#42-numeric-operations)
    - [4.3. The Boolean type](#43-the-boolean-type)
    - [4.4. The character type](#44-the-character-type)
    - [4.5. Compound types](#45-compound-types)
      - [4.5.1. The tuple type](#451-the-tuple-type)
      - [4.5.2. The array type](#452-the-array-type)
  - [5. Functions](#5-functions)
    - [5.1. Parameters](#51-parameters)
    - [5.2. Statements and expressions](#52-statements-and-expressions)
    - [5.3. Function return values](#53-function-return-values)
  - [6. Control flow](#6-control-flow)
    - [6.1. if expressions](#61-if-expressions)
    - [6.2. Repetition with loops](#62-repetition-with-loops)
      - [6.2.1. Repeating code with loop](#621-repeating-code-with-loop)
      - [6.2.2. Conditional loops with while](#622-conditional-loops-with-while)
      - [6.2.3. Looping through a collection with for](#623-looping-through-a-collection-with-for)
  - [7. Ownership](#7-ownership)
    - [7.1. References and borrowing](#71-references-and-borrowing)
    - [7.2. The slice type](#72-the-slice-type)
  - [8. Structs](#8-structs)
    - [8.1. Methods](#81-methods)
  - [9. Enums](#9-enums)
  - [10. The match control flow construct](#10-the-match-control-flow-construct)
  - [11. Control flow with if let](#11-control-flow-with-if-let)
  - [12. Generics and traits](#12-generics-and-traits)
    - [12.1. Generics in structs](#121-generics-in-structs)
    - [12.2. Generics in enums](#122-generics-in-enums)
    - [12.3. Generics in methods](#123-generics-in-methods)
    - [12.4. const generics (an important feature introduced in Rust 1.51)](#124-const-generics-an-important-feature-introduced-in-rust-151)
      - [12.4.1 const generic expressions](#1241-const-generic-expressions)
    - [12.5. Performance of generics](#125-performance-of-generics)
    - [12.6. Traits](#126-traits)
      - [12.6.1. Implementing a trait for a type](#1261-implementing-a-trait-for-a-type)
      - [12.6.2. Where traits are defined and implemented (the orphan rule)](#1262-where-traits-are-defined-and-implemented-the-orphan-rule)
      - [12.6.3. Default implementations](#1263-default-implementations)
      - [12.6.4. Traits as function parameters](#1264-traits-as-function-parameters)
      - [12.6.5. Trait bounds](#1265-trait-bounds)
      - [12.6.6. Multiple bounds](#1266-multiple-bounds)
      - [12.6.7. where clauses](#1267-where-clauses)
      - [12.6.8. Conditionally implementing methods or traits with trait bounds](#1268-conditionally-implementing-methods-or-traits-with-trait-bounds)
      - [12.6.9. impl Trait in return position](#1269-impl-trait-in-return-position)
      - [12.6.10. Implementing + for a custom type](#12610-implementing--for-a-custom-type)
    - [12.7. Trait objects](#127-trait-objects)
  - [13. Vectors](#13-vectors)
    - [13.1. Creating a vector](#131-creating-a-vector)
    - [13.2. Updating a vector](#132-updating-a-vector)
    - [13.3. Reading elements of a vector](#133-reading-elements-of-a-vector)
    - [13.4. Borrowing several vector elements at once](#134-borrowing-several-vector-elements-at-once)
    - [13.5. Iterating over the elements of a vector](#135-iterating-over-the-elements-of-a-vector)
    - [13.6. Storing elements of different types](#136-storing-elements-of-different-types)
    - [13.7. Common vector methods](#137-common-vector-methods)
    - [13.8. Sorting vectors](#138-sorting-vectors)
  - [14. HashMap](#14-hashmap)
    - [14.1. Creating a HashMap](#141-creating-a-hashmap)
    - [14.2. Ownership transfer](#142-ownership-transfer)
    - [14.3. Querying a HashMap](#143-querying-a-hashmap)
    - [14.4. Updating values in a HashMap](#144-updating-values-in-a-hashmap)
    - [14.5. Hash functions](#145-hash-functions)
  - [15. Lifetimes](#15-lifetimes)
    - [15.1. Dangling pointers and lifetimes](#151-dangling-pointers-and-lifetimes)
    - [15.2. The borrow checker](#152-the-borrow-checker)
    - [15.3. Lifetimes in functions](#153-lifetimes-in-functions)
  - [16. Methods](#16-methods)
    - [16.1. Defining methods](#161-defining-methods)
    - [16.2. self, &self and &mut self](#162-self-self-and-mut-self)
    - [16.3. Methods with the same name as a field](#163-methods-with-the-same-name-as-a-field)
    - [16.4. Methods with more parameters](#164-methods-with-more-parameters)
    - [16.5. Associated functions](#165-associated-functions)
    - [16.6. Multiple impl blocks](#166-multiple-impl-blocks)
    - [16.7. Implementing methods on enums](#167-implementing-methods-on-enums)
  - [17. Creating macros with macro\_rules!](#17-creating-macros-with-macro_rules)
    - [17.1. Designators](#171-designators)
    - [17.2. Overloading](#172-overloading)
    - [17.3. Repetition](#173-repetition)
    - [17.4. DRY (Don't Repeat Yourself)](#174-dry-dont-repeat-yourself)
    - [17.5. DSL (domain-specific languages)](#175-dsl-domain-specific-languages)
    - [17.6. Variadic interfaces](#176-variadic-interfaces)
  - [18. Error handling](#18-error-handling)
    - [18.1. panic](#181-panic)
    - [18.2. Option and unwrap](#182-option-and-unwrap)
    - [18.3. Unpacking Options with ?](#183-unpacking-options-with-)
    - [18.4. Combinators: map](#184-combinators-map)
    - [18.5. Combinators: and\_then](#185-combinators-and_then)
    - [18.6. Result](#186-result)
    - [18.7. map for Result](#187-map-for-result)
    - [18.8. Aliases for Result](#188-aliases-for-result)
    - [18.9. Early returns](#189-early-returns)
    - [18.10. Introducing ?](#1810-introducing-)
    - [18.11. The try! macro](#1811-the-try-macro)
    - [18.12. Handling multiple error types](#1812-handling-multiple-error-types)
    - [18.13. Pulling Results out of Options](#1813-pulling-results-out-of-options)
    - [18.14. Defining an error type](#1814-defining-an-error-type)
    - [18.15. Boxing errors](#1815-boxing-errors)
    - [18.16. Other uses of ?](#1816-other-uses-of-)
    - [18.17. Wrapping errors](#1817-wrapping-errors)
    - [18.18. Iterating over Results](#1818-iterating-over-results)
      - [18.18.1. Ignoring failed items with filter\_map()](#18181-ignoring-failed-items-with-filter_map)
      - [18.18.2. Failing the entire operation with collect()](#18182-failing-the-entire-operation-with-collect)
      - [18.18.3. Collecting all valid values and errors with partition()](#18183-collecting-all-valid-values-and-errors-with-partition)
  - [x. Modules](#x-modules)
    - [x.1. Packages and crates](#x1-packages-and-crates)
    - [x.2 Defining modules to control scope and privacy](#x2-defining-modules-to-control-scope-and-privacy)
    - [x.3 Library packages](#x3-library-packages)
      - [x.3.1. Package layout](#x31-package-layout)
## 0. Repository layout

```text
rust-learn/
├── README.md        these study notes
├── docs/images/     images used in the notes
├── examples/        small examples grouped by topic; each folder is a standalone Cargo project
└── projects/        complete, larger projects
```

### 0.1. Examples

| Folder | Topic | Examples | Related notes |
| ---- | ---- | ---- | -------- |
| [`01-basics`](examples/01-basics) | Basics: formatted output, enums and `impl` | [`format-demo`](examples/01-basics/format-demo), [`impl-demo`](examples/01-basics/impl-demo), [`type-demo`](examples/01-basics/type-demo) | [3. Basic syntax](#3-basic-syntax), [16. Methods](#16-methods) |
| [`02-collections`](examples/02-collections) | Collections and linked lists | [`vec-demo`](examples/02-collections/vec-demo), [`hashmap-demo`](examples/02-collections/hashmap-demo), [`hashset-demo`](examples/02-collections/hashset-demo), [`lists`](examples/02-collections/lists), [`linked_list`](examples/02-collections/linked_list), [`reverse_list_demo`](examples/02-collections/reverse_list_demo) | [13. Vector](#13-vectors), [14. HashMap](#14-hashmap) |
| [`03-modules`](examples/03-modules) | Modules and libraries | [`mod-demo`](examples/03-modules/mod-demo), [`module-demo`](examples/03-modules/module-demo), [`my-lib`](examples/03-modules/my-lib) | [x. Modules](#x-modules) |
| [`04-errors`](examples/04-errors) | Error handling: `Result`, custom errors | [`error_demo`](examples/04-errors/error_demo), [`result-demo`](examples/04-errors/result-demo) | [18. Error handling](#18-error-handling) |
| [`05-generics-traits`](examples/05-generics-traits) | Generics, traits, trait objects, lifetimes | [`genericity_demo`](examples/05-generics-traits/genericity_demo), [`trait-demo`](examples/05-generics-traits/trait-demo), [`trait-object`](examples/05-generics-traits/trait-object), [`lifetime-demo`](examples/05-generics-traits/lifetime-demo) | [12. Generics and traits](#12-generics-and-traits), [15. Lifetimes](#15-lifetimes) |
| [`06-smart-pointers`](examples/06-smart-pointers) | Smart pointers: `Box`, `Arc` | [`mut-demo`](examples/06-smart-pointers/mut-demo), [`rc-demo`](examples/06-smart-pointers/rc-demo) | — |
| [`07-concurrency`](examples/07-concurrency) | Concurrency: threads, channels, a thread-pool web server | [`thread-demo`](examples/07-concurrency/thread-demo), [`channel-demo`](examples/07-concurrency/channel-demo), [`web-server`](examples/07-concurrency/web-server) | — |
| [`08-advanced`](examples/08-advanced) | Macros and unsafe | [`macro_rules_demo`](examples/08-advanced/macro_rules_demo), [`unsafe_demo`](examples/08-advanced/unsafe_demo) | [17. macro_rules!](#17-creating-macros-with-macro_rules) |
| [`09-testing-io`](examples/09-testing-io) | Testing, file I/O, command-line arguments, minigrep | [`test_demo`](examples/09-testing-io/test_demo), [`file-demo`](examples/09-testing-io/file-demo), [`args-demo`](examples/09-testing-io/args-demo), [`minigrep`](examples/09-testing-io/minigrep) | — |
| [`10-web-axum`](examples/10-web-axum) | Axum web: CORS, SSE, WebSocket chat, path-parameter error handling | [`cors`](examples/10-web-axum/cors), [`sse`](examples/10-web-axum/sse), [`chart`](examples/10-web-axum/chart), [`customize-path-rejection`](examples/10-web-axum/customize-path-rejection) | — |

Each example is a standalone Cargo project; run it from its folder, for example:

```sh
cd examples/07-concurrency/web-server
cargo run          # serves hello.html on http://127.0.0.1:7878, exits after two requests
```

### 0.2. Projects

- [`smart-access-control`](projects/smart-access-control): a smart access control system (software simulation only).
  Rust backend (Axum, Tokio, SQLx, PostgreSQL, layered following Clean Architecture) plus a
  React dashboard, with an access decision engine, JWT login with refresh-token rotation,
  live events over WebSocket, simulated door controllers, Prometheus metrics and Docker deployment.
  Docs: [English](projects/smart-access-control/README.md) |
  [中文](projects/smart-access-control/README.zh-CN.md)

## 1. Installing Rust (Mac)

```text
brew install rustup-init
```

Then run

```text
rustup-init
```

## 2. Cargo

Cargo is Rust's build system and package manager.

### 2.1. Creating a project with Cargo

```text
cargo new hello_cargo
```

Go into the hello_cargo directory and list its files. Cargo has generated two files and one directory: a Cargo.toml file, a src directory, and a main.rs file inside src.  

> Filename: Cargo.toml: written in TOML (Tom's Obvious, Minimal Language), the format of Cargo's configuration files.

```toml
[package]
name = "hello-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
```

The first line, [package], is a section heading: the statements below it configure a package. As we add more information to this file, we will add other sections.  

The last line, [dependencies], starts the section that lists the project's dependencies. In Rust, packages of code are called crates.  

#### 2.1.1. Dependencies from crates.io

```text
[dependencies]
time = "0.1.12"
```

The string "0.1.12" is a semver version number of the form "x.y.z", where x is the major version, y the minor version and z the patch. From left to right, each part has a smaller impact; a patch update is harmless and does not break API compatibility.

"0.1.12" has no extra symbol; semantically it is the same as "^0.1.12" with a caret, and both select a very specific version.

>| ^ version requirements  

Unlike the plain "0.1.12" before, ^ specifies a range of versions, and the highest version within that range is used.  

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

>| ~ version requirements  

~ specifies a minimal version:

```text
~1.2.3  := >=1.2.3, <1.3.0
~1.2    := >=1.2.0, <1.3.0
~1      := >=1.0.0, <2.0.0
```

>| * wildcard

Allows any number in the position of the *:

```text
*     := >=0.0.0
1.*   := >=1.0.0, <2.0.0
1.2.* := >=1.2.0, <1.3.0
```

>| Comparison operators  

These version rules apply only to crates.io and registries built on it (such as the USTC mirror); other registries (such as GitHub) have their own rules.  

Comparison operators specify a range of versions or an exact version:

```text
>= 1.2.0
> 1
< 2
= 1.2.3
```

Comparisons can also be combined, separated by commas:

```text
>= 1.2, < 1.5
```

#### 2.1.2. Dependencies from other registries

To use a registry other than crates.io, configure $HOME/.cargo/config.toml (under $CARGO_HOME) and add the new registry. There are two ways to do this.  

>| Use the USTC registry for faster downloads

To add a registry alongside crates.io, add the following to .cargo/config.toml:

```text
[registries]
ustc = { index = "https://mirrors.ustc.edu.cn/crates.io-index/" }
```

With this approach, dependencies are declared differently in the project's Cargo.toml:

```text
[dependencies]
time = {  registry = "ustc" }
```

After this change, the first build may take longer, because the index of the ustc registry has to be downloaded. The main drawback of this approach is that every dependency has to name its registry: time = { registry = "ustc" }.  

>| Replace the default crates.io with the new registry  

Replace the source source.crates-io with ustc, then give the address of the ustc source in the second part.  

Note: if you publish a package to crates.io, its dependencies must also be on crates.io.  

```text
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```

#### 2.1.3. Git repositories as dependencies

```text
[dependencies]
regex = { git = "https://github.com/rust-lang/regex" }
```

Since no version is given, Cargo assumes the latest commit on the master or main branch. Use rev, tag or branch to choose what to pull. For example, the following pulls the latest commit on the next branch:

```text
[dependencies]
regex = { git = "https://github.com/rust-lang/regex", branch = "next" }
```

Anything that is not a tag or branch can be selected with rev, for example the hash of a recent commit: rev = "4c59b707", or a named reference provided by the remote repository: rev = "refs/pull/493/head".  

Once a git dependency has been fetched, its version is recorded and locked in Cargo.lock, so later commits in the repository are not pulled automatically unless you upgrade with cargo update. Note that if the lock is deleted, Cargo fetches again according to the address and version in Cargo.toml; if that version is wrong, you may pull an incompatible new version!

#### 2.1.4. Local dependencies by path

Local dependencies are internal packages of the same project. For example, suppose we have a hello_world project (package) and create a new package in its root directory:

```text
#  in the hello_world/ directory
$ cargo new hello_utils
```

The new hello_utils folder sits next to src and Cargo.toml. Now edit Cargo.toml so that the hello_world project uses the new package:

```text
[dependencies]
hello_utils = { path = "hello_utils" }
# this path also works
# hello_utils = { path = "./hello_utils" }
# hello_utils = { path = "../hello_world/hello_utils" }
```

#### 2.1.5. Platform-specific dependencies

Include dependencies only on specific platforms:

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

Logical operators work too: here openssl is included only when the operating system is not unix.

```text
[target.'cfg(not(unix))'.dependencies]
openssl = "1.0.1"
```

#### 2.1.6. [dev-dependencies]

To add libraries that are only needed for tests, similar to devDependencies in a Node.js package.json, add a [dev-dependencies] section to Cargo.toml:

```text
[dev-dependencies]
tempdir = "0.3"
```

These dependencies are only used when running tests, examples and benchmarks. Also, if package A depends on B, and B depends on C through [dev-dependencies], A does not depend on C.

Platform-specific test dependencies are possible too:

```text
[target.'cfg(unix)'.dev-dependencies]
mio = "0.0.1"
```

#### 2.1.7. [build-dependencies]

Dependencies used only by the build script:

```text
[build-dependencies]
cc = "1.0.3"
```

Platform-specific build dependencies:

[target.'cfg(unix)'.build-dependencies]
cc = "1.0.3"

#### 2.1.8 Choosing features

If a dependency offers optional features, you can choose which ones to use:

```text
[dependencies.awesome]
version = "1.3.5"
default-features = false # do not include the default features; list the wanted ones as below
features = ["secure-password", "civet"]
```

#### 2.1.9. Renaming dependencies in Cargo.toml

Avoids writing use foo as bar in Rust code  
Depend on several versions of the same package  
Depend on packages with the same name from different registries  

Use the package key provided by Cargo:

```text
[package]
name = "mypackage"
version = "0.0.1"

[dependencies]
foo = "0.1"
bar = { git = "https://github.com/example/project", package = "foo" }
baz = { version = "0.1", registry = "custom", package = "foo" }
```

### 2.2. Building and running a Cargo project

> cargo build: creates an executable at target/debug/hello_rust

The first cargo build also makes Cargo create a new file in the project root: Cargo.lock.  

> cargo run compiles and runs the executable in one command. Instead of remembering to run cargo build and then the executable by its full path, cargo run does the same and is much more convenient, so most developers use cargo run.  

> The cargo check command quickly checks that the code compiles, without producing an executable.

### 2.3. Cargo.toml format

Cargo.toml is also called the manifest. Its format is TOML, and every manifest consists of the following parts:

- cargo-features — features only available on nightly  
- [package] — metadata of the package  
  + name — name  
  + version — version  
  + authors — authors  
  + edition — Rust edition.  
  + rust-version — minimum supported Rust version  
  + description — description  
  + documentation — documentation URL  
  + readme — path to the README file  
  + homepage - home page URL  
  + repository — URL of the source repository  
  + license — open-source license.  
  + license-file — path to the license file.  
  + keywords — keywords of the package  
  + categories — categories of the package  
  + workspace — path to the workspace  
  + build — path to the build script  
  + links — name of the native library being linked  
  + exclude — files excluded when publishing  
  + include — files included when publishing  
  + publish — prevents the package from being published  
  + metadata — extra settings for external tools  
  + default-run — the default binary used by [cargo run]  
  + autobins — disables automatic discovery of binaries  
  + autoexamples — disables automatic discovery of examples  
  + autotests — disables automatic discovery of tests  
  + autobenches — disables automatic discovery of benches  
  + resolver — sets the dependency resolver  
- Cargo target list: (see the target configuration for details)  
  + [lib] — library target settings.  
  + [[bin]] — binary target settings.  
  + [[example]] — example target settings.  
  + [[test]] — test target settings.  
  + [[bench]] — benchmark target settings.  
- Dependency tables:  
  + [dependencies] — the package's dependencies  
  + [dev-dependencies] — dependencies for examples, tests and benchmarks  
  + [build-dependencies] — dependencies for build scripts  
  + [target] — platform-specific dependencies  
  + [badges] — status shown on registries (such as crates.io), e.g. maintenance status: actively developed, looking for a maintainer, deprecated  
  + [features] — features, used for conditional compilation  
  + [patch] — the recommended way to override dependencies  
  + [replace] — the deprecated way to override dependencies.  
  + [profile] — compiler settings and optimizations  
  + [workspace] — workspace definition  

### 2.4. Release builds

Run cargo build --release and test with the executable under target/release.  

## 3. Basic syntax

### 3.1. Variables

Put mut before a variable name to make it mutable.

```rs
fn main() {
    let mut x = 5;
    println!("The value of x is: {x}");
    x = 6;
    println!("The value of x is: {x}");
}
```

### 3.2. Constants

Constants are values bound to a name that are not allowed to change; mut is not allowed on constants.

```rs
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

### 3.3. Shadowing

A variable can be shadowed by declaring a new one with the same name, using the let keyword again, as many times as needed.

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

Another difference between mut and shadowing: using let again actually creates a new variable, which can change the type of the value while reusing the name.  

## 4. Data types

### 4.1. Scalar types

A scalar type represents a single value. Rust has four primary scalar types: integers, floating-point numbers, Booleans and characters.

#### 4.1.1. Integer types

Numbers without a fractional part.  

| Length | Signed | Unsigned |
|  ----  | ----  |
| 8-bit |  i8 | u8 |
| 16-bit | i16 | u16 |
| 32-bit | i32 | u32 |
| 64-bit | i64 | u64 |
| 128-bit | i128 | u128 |
| arch | isize | usize |

Signed and unsigned refer to whether the number can be negative.  

Each signed variant can store numbers from -2^(n-1) to 2^(n-1) - 1 inclusive, where n is the number of bits the variant uses. So i8 can store numbers from -2^7 to 2^7 - 1, i.e. from -128 to 127. Unsigned variants can store numbers from 0 to 2^n - 1, so u8 can store numbers from 0 to 2^8 - 1, i.e. from 0 to 255.  

The isize and usize types depend on the architecture of the computer the program runs on: 64 bits on a 64-bit architecture, 32 bits on a 32-bit architecture.  

| Number literal | Example |
| Decimal | 98_222 |
| Hex | 0xff |
| Octal | 0o77 |
| Binary | 0b1111_0000 |
| Byte (u8 only) | b'A' |

If you're unsure, Rust's defaults are generally a good place to start: integer types default to i32. isize or usize are mainly used for indexing collections.  

> Integer overflow

Say you have a u8, which can hold values from zero to 255. What happens if you change it to 256? This is called "integer overflow", and it results in one of two behaviors. When compiling in debug mode, Rust checks for it and makes the program panic, the term Rust uses when a program exits with an error.  

In release builds, Rust does not check for overflow and instead performs two's complement wrapping. In short, 256 becomes 0, 257 becomes 1, and so on. Relying on integer overflow is considered an error, even though this behavior can happen. If you really need it, the standard library has a type that provides it explicitly: Wrapping. To handle possible overflow explicitly, use these methods that the standard library provides on primitive numeric types:  

wrap in all modes with the wrapping_* methods, such as wrapping_add  
return None on overflow with the checked_* methods  
return the value and a Boolean indicating overflow with the overflowing_* methods  
saturate at the minimum or maximum value with the saturating_* methods  

#### 4.1.2. Floating-point types

Rust's floating-point types are f32 and f64, which are 32 and 64 bits in size. The default is f64, because on modern CPUs it is roughly as fast as f32 but more precise. All floating-point types are signed.  

### 4.2. Numeric operations

Addition, subtraction, multiplication, division and remainder. Integer division rounds down to the nearest integer.  

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

### 4.3. The Boolean type

A Boolean in Rust has two possible values: true and false. The Boolean type is written bool.  

```rs
fn main() {
    let t = true;
    let f: bool = false;
}
```

### 4.4. The character type

char literals use single quotes; string literals use double quotes.

### 4.5. Compound types

Compound types group multiple values into one type. Rust has two primitive compound types: tuples and arrays.

#### 4.5.1. The tuple type

A tuple is the general way of grouping values of different types into one compound type. Tuples have a fixed length: once declared, they cannot grow or shrink.  

```rs
fn main() {
  let tup: (i32, f64, u8) = (500, 6.4, 1);
}
```

Pattern matching can be used to destructure a tuple value

```rs
fn main() {
  let tup = (500, 6.4, 1);
  let (x, y, z) = tup;
  println!("The value of y is: {y}");
}
```

A tuple element can be accessed directly with a period (.) followed by its index.

```rs
fn main() {
  let x: (i32, f64, u8) = (500, 6.4, 1);
  let five_hundred = x.0;
  let six_point_four = x.1;
  let one = x.2;
}
```

The tuple without any values has a special name: the unit tuple. Both the value and its type are written () and represent an empty value or an empty return type. Expressions implicitly return the unit value if they don't return any other value.

#### 4.5.2. The array type

Unlike a tuple, every element of an array must have the same type. Arrays in Rust have a fixed length.  

An array is not as flexible as a vector. A vector is a similar collection type provided by the standard library that is allowed to grow and shrink. If you're unsure whether to use an array or a vector, you should probably use a vector.  

```rs
let a: [i32; 5] = [1, 2, 3, 4, 5];
```

Here, i32 is the type of each element. After the semicolon, the number 5 says the array contains five elements.

You can also create an array where every element has the same value by giving the initial value, a semicolon, and the length in square brackets:

```rs
let a = [3; 5];
```

The array named a will contain 5 elements, all initially set to 3. This is the same as writing let a = [3, 3, 3, 3, 3]; but more concise.

> Accessing array elements

Use indexing to access the elements of an array

```rs
fn main() {
  let a = [1, 2, 3, 4, 5];

  let first = a[0];
  let second = a[1];
}
```

## 5. Functions

Rust code uses snake case for function and variable names: all letters are lowercase and underscores separate words.

```rs
fn main() {
  println!("Hello, world!");
  another_function();
}

fn another_function() {
  println!("Another function.");
}
```

Rust doesn't care where functions are defined, only that they are defined in a scope visible to the caller.  

### 5.1. Parameters

In function signatures, you must declare the type of each parameter.

```rs
fn main() {
  print_labeled_measurement(5, 'h');
}

fn print_labeled_measurement(value: i32, unit_label: char) {
  println!("The measurement is: {value}{unit_label}");
}
```

### 5.2. Statements and expressions

> Statements are instructions that perform some action and do not return a value.  

let y = 6; is a statement.  

```rs
fn main() {
  let y = 6;
}
```

> Expressions evaluate to a resulting value.

Calling a function is an expression. Calling a macro is an expression. A new block scope created with curly braces is also an expression.

```rs
fn main() {
  let y = {
    let x = 3;
    x + 1
  };

  println!("The value of y is: {y}");
}
```

The expression is a block whose value is 4.

```rs
{
  let x = 3;
  x + 1
}
```

### 5.3. Function return values

Declare the return type after an arrow (->).  
The return value of a function is the value of the final expression in its body. You can return early with the return keyword and a value, but most functions implicitly return the last expression.

```rs
fn main() {
    let x = plus_one(5);

    println!("The value of x is: {x}");
}

fn plus_one(x: i32) -> i32 {
    x + 1
}
```

Running the code prints The value of x is: 6. But if you put a semicolon at the end of the line containing x + 1, turning it from an expression into a statement, you get an error.

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

"mismatched types" reveals the core problem. The definition of plus_one says it returns an i32, but statements don't evaluate to a value, which is expressed by the unit type (). Because returning nothing contradicts the function definition, you get an error. In the output, Rust gives a hint that may help fix it: it suggests removing the semicolon, which would fix the error.

## 6. Control flow

The most common constructs for controlling the flow of execution in Rust code are if expressions and loops.  

### 6.1. if expressions

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

> Using if in a let statement

```rs
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 };

    println!("The value of number is: {number}"); // 5
}
```

### 6.2. Repetition with loops

Rust has three kinds of loops: loop, while and for.

#### 6.2.1. Repeating code with loop

Use the break keyword to tell the program when to stop the loop.  
The continue keyword in a loop tells the program to skip the rest of the current iteration and go to the next one.  

> Returning values from loops

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

> Loop labels to disambiguate between multiple loops

With nested loops, break and continue apply to the innermost loop at that point. You can optionally give a loop a loop label and use it with break or continue, so that those keywords apply to the labeled loop instead of the innermost one.

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

#### 6.2.2. Conditional loops with while

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

#### 6.2.3. Looping through a collection with for

```rs
fn main() {
  let a = [10, 20, 30, 40, 50];

  for element in a {
      println!("the value is: {element}");
  }
}
```

## 7. Ownership

Every program has to manage the way it uses the computer's memory while running. Some languages have garbage collection that constantly looks for memory no longer in use while the program runs; in others, the programmer must explicitly allocate and free memory. Rust takes a third approach: memory is managed through a system of ownership, with a set of rules that the compiler checks at compile time. If any of the rules are violated, the program won't compile. None of the features of ownership slow down the program while it runs.

> Ownership rules

1. Each value in Rust has a variable that is called its owner.
2. There can only be one owner at a time.
3. When the owner (variable) goes out of scope, the value is dropped.

> Variable scope

A scope is the range within a program for which an item is valid.

> Memory and allocation

Memory is automatically freed once the variable that owns it goes out of scope.

1. Ways variables and data interact (1): move

```rs
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;

    println!("{}, world!", s1);
}
```

To ensure memory safety, after let s2 = s1 Rust considers s1 no longer valid, so Rust doesn't need to free anything when s1 goes out of scope.

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

If you've heard the terms shallow copy and deep copy in other languages, copying the pointer, length and capacity without copying the data probably sounds like a shallow copy. But because Rust also invalidates the first variable, this operation is known as a move rather than a shallow copy.  

Rust never automatically creates "deep" copies of your data. Therefore, any automatic copying can be assumed to be inexpensive in terms of runtime performance.

2. Ways variables and data interact (2): clone

```rs
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();

    println!("s1 = {}, s2 = {}", s1, s2);
}
```

3. Stack-only data: copy

```rs
fn main() {
    let x = 5;
    let y = x;

    println!("x = {}, y = {}", x, y);
}
```

As a general rule, any group of simple scalar values can implement Copy, and nothing that requires allocation or is some form of resource can implement Copy. Here are some Copy types:

- All integer types, such as u32.
- The Boolean type, bool, with values true and false.
- All floating-point types, such as f64.
- The character type, char.
- Tuples, if they only contain types that also implement Copy. For example, (i32, i32) implements Copy, but (i32, String) does not.

> Ownership and functions

```rs
fn main() {
    let s = String::from("hello");  // s comes into scope

    takes_ownership(s);             // s's value moves into the function...
                                    // ... and so is no longer valid here

    let x = 5;                      // x comes into scope

    makes_copy(x);                  // x would move into the function,
                                    // but i32 is Copy,
                                    // so it's okay to still use x afterward

} // Here, x goes out of scope, then s. But because s's value was moved,
  // nothing special happens

fn takes_ownership(some_string: String) { // some_string comes into scope
    println!("{}", some_string);
} // Here, some_string goes out of scope and `drop` is called.
  // The backing memory is freed

fn makes_copy(some_integer: i32) { // some_integer comes into scope
    println!("{}", some_integer);
} // Here, some_integer goes out of scope. Nothing special happens
```

> Return values and scope

Returning values can also transfer ownership.

```rs
fn main() {
    let s1 = gives_ownership();         // gives_ownership moves its return
                                        // value into s1

    let s2 = String::from("hello");     // s2 comes into scope

    let s3 = takes_and_gives_back(s2);  // s2 is moved into
                                        // takes_and_gives_back, which also
                                        // moves its return value into s3
} // Here, s3 goes out of scope and is dropped. s2 also goes out of scope, but was moved,
  // so nothing happens. s1 goes out of scope and is dropped

fn gives_ownership() -> String {             // gives_ownership will move its
                                             // return value into the function
                                             // that calls it

    let some_string = String::from("yours"); // some_string comes into scope.

    some_string                              // some_string is returned and 
                                             // moves out to the calling function
                                             // 
}

// takes_and_gives_back takes a String and returns it
fn takes_and_gives_back(a_string: String) -> String { // a_string comes into scope

    a_string  // a_string is returned and moves out to the calling function
}
```

### 7.1. References and borrowing

> References

A reference is like a pointer in that it is an address we can follow to access data stored at that address, which is owned by some other variable. Unlike a pointer, a reference is guaranteed to point to a valid value of a particular type.

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

The & signs are references; they let you use a value without taking ownership of it.

![Reference](/docs/images/1.svg)

The &s1 syntax creates a reference that refers to the value of s1 but does not own it. Because it does not own it, the value it points to is not dropped when the reference stops being used.

```rs
let s1 = String::from("hello");
let len = calculate_length(&s1);
```

> Borrowing

The action of creating a reference is called borrowing.

```rs
fn main() {
    let s = String::from("hello");

    change(&s);
}

fn change(some_string: &String) {
    some_string.push_str(", world");
}
```

Modifying a borrowed value through a plain reference is not allowed.

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

> Mutable references

A mutable reference lets us modify a borrowed value:

Fixing the error in the code above: 

```rs
fn main() {
  let mut s = String::from("hello");
  change(&mut s);
}

fn change(some_string: &mut String) {
  some_string.push_str(", world");
}
```

Change s to mut. Then create a mutable reference with &mut s where change is called, and update the function signature to accept a mutable reference: some_string: &mut String. This makes it very clear that change will mutate the value it borrows.

Mutable references have one big restriction: you can have only one mutable reference to a particular piece of data at a time. This code, which tries to create two mutable references to s, fails:

```rs
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;

    println!("{}, {}", r1, r2);
}
```

Error:

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

Curly brackets can create a new scope, allowing multiple mutable references, just not simultaneous ones:

```rs
fn main() {
    let mut s = String::from("hello");
    {
        let r1 = &mut s;
    } // r1 goes out of scope here, so we can make a new reference with no problems
    let r2 = &mut s;
}
```

You also cannot have a mutable reference while you have an immutable one to the same value. Users of an immutable reference don't expect the value to suddenly change out from under them! Multiple immutable references are fine, though, because no one who is only reading the data can affect anyone else's reading of it.

```rs
fn main() {
  let mut s = String::from("hello");

  let r1 = &s; // no problem
  let r2 = &s; // no problem
  let r3 = &mut s; // BIG PROBLEM

  println!("{}, {}, and {}", r1, r2, r3);
}
```

A reference's scope starts where it is introduced and continues until the last time it is used. For example, this code compiles, because the last use of the immutable references (println!) happens before the mutable reference is introduced:

```rs
fn main() {
  let mut s = String::from("hello");

  let r1 = &s; // no problem
  let r2 = &s; // no problem
  println!("{} and {}", r1, r2);
  // r1 and r2 are not used after this point

  let r3 = &mut s; // no problem
  println!("{}", r3);
}
```

> Dangling references

A dangling pointer references memory that may have been given to someone else. In Rust, by contrast, the compiler guarantees that references will never be dangling: if you have a reference to some data, the compiler ensures the data will not go out of scope before the reference does.  

```rs
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String {
    let s = String::from("hello");

    &s
}
```

Error:

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

The solution is to return the String directly:

```rs
fn no_dangle() -> String {
    let s = String::from("hello");

    s
}
```

### 7.2. The slice type

A string slice is a reference to part of a String

```rs
let s = String::from("hello world");
let hello = &s[0..5];
let world = &s[6..11];
```

A slice is created with a range in brackets, [starting_index..ending_index], where starting_index is the first position in the slice and ending_index is one more than the last position. Internally, the slice data structure stores the starting position and the length, which is ending_index minus starting_index. So for let world = &s[6..11];, world is a slice that contains a pointer to index 6 of s and a length of 5.

With Rust's .. range syntax, if you want to start at index 0 you can drop the value before the two periods. These two statements are equivalent:

```rs
let s = String::from("hello");

let slice = &s[0..2];
let slice = &s[..2];
```

If the slice includes the last byte of the String, you can drop the trailing number too.

```rs
let s = String::from("hello");

let len = s.len();

let slice = &s[3..len];
let slice = &s[3..];
```

Drop both values to take a slice of the entire string.

```rs
let slice = &s[0..len];
let slice = &s[..];
```

Writing a function that takes a string slice instead of a reference to a String makes our API more general without losing any functionality:

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

  // `first_word` works on slices of `String`s, whether partial or whole
  let word = first_word(&my_string[0..6]);
  let word = first_word(&my_string[..]);
  // `first_word` also works on references to `String`s, which are
  // equivalent to whole slices of `String`s
  let word = first_word(&my_string);

  let my_string_literal = "hello world";

  // `first_word` works on slices of string literals, whether partial or whole
  let word = first_word(&my_string_literal[0..6]);
  let word = first_word(&my_string_literal[..]);

  // Because string literals **are** string slices already,
  // this works too, without the slice syntax!
  let word = first_word(my_string_literal);
}
```

## 8. Structs

Structs name each piece of data so it's clear what the values mean. Thanks to these names, structs are more flexible than tuples: you don't have to rely on the order of the data to specify or access the values of an instance.

A struct that stores information about a user account:

```rs
struct User {
  active: bool,
  username: String,
  email: String,
  sign_in_count: u64,
}
```

Usage: create an instance of the struct by giving each field a concrete value.  

An instance starts with the name of the struct, followed by curly brackets containing key: value pairs for the fields.  

Declaring a particular user:

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

To get a specific value from a struct, use dot notation. For the user's email address, use user1.email.  
If the instance is mutable, change a value by using dot notation and assigning to the field.  

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

The build_user function returns a User instance with the given email and username.

```rs
fn build_user(email: String, username: String) -> User {
  User {
    email: email,
    username: username,
    active: true,
    sign_in_count: 1,
  }
}

// shorthand
fn build_user(email: String, username: String) -> User {
  User {
    email,
    username,
    active: true,
    sign_in_count: 1,
  }
}
```

The .. syntax specifies that the remaining fields not explicitly set should have the same values as the fields in the given instance.

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

// .. syntax
let user2 = User {
  email: String::from("another@example.com"),
  ..user1
};
```

> Defining tuple structs

Start with the struct keyword and the struct name, followed by the types in the tuple.  

```rs
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
  let black = Color(0, 0, 0);
  let origin = Point(0, 0, 0);
}
```

> Unit-like structs without any fields

These are called unit-like structs because they behave similarly to (), the unit type mentioned in the "tuple type" section.

```rs
struct AlwaysEqual;

fn main() {
  let subject = AlwaysEqual;
}
```

### 8.1. Methods

Methods are similar to functions: they are declared with the fn keyword and a name, can have parameters and a return value, and contain code that runs when the method is called. Unlike functions, methods are defined within the context of a struct (or an enum or a trait object), and their first parameter is always self, which represents the struct instance the method is called on.

Defining an area method on the Rectangle struct

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

In the signature of area, &self is used instead of rectangle: &Rectangle; &self is actually short for self: &Self.  

The & in front of self indicates that the method borrows the Self instance.  

## 9. Enums

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

A Message enum whose variants each store different amounts and types of values

```rs
enum Message {
  Quit,
  Move { x: i32, y: i32 },
  Write(String),
  ChangeColor(i32, i32, i32),
}
```

- Quit has no data associated with it at all.
- Move has named fields, like a struct.
- Write includes a single String.
- ChangeColor includes three i32 values.

## 10. The match control flow construct

Compares a value against a series of patterns and runs the code of the pattern that matches. Patterns can be made up of literal values, variable names, wildcards and many other things.  

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

## 11. Control flow with if let

The if let syntax takes a pattern and an expression separated by an equal sign. It works the same way as a match, where the expression is given to the match and the pattern is its first arm

```rs
let config_max = Some(3u8);
match config_max {
  Some(max) => println!("The maximum is configured to be {}", max),
  _ => (),
}

// same as above
let config_max = Some(3u8);
if let Some(max) = config_max {
   println!("The maximum is configured to be {}", max);
}
```

## 12. Generics and traits

You can name a generic parameter anything, but by convention T (the first letter of "type") is the usual first choice.  

A generic parameter must be declared before it is used:

```rust
fn largest<T>(list: &[T]) -> T {
```

### 12.1. Generics in structs

Field types in a struct can also be generic. Two things to note:  

- Declare first: as with generic functions, the generic parameter must be declared before use, as Point<T>; then T can be used in place of concrete types in the struct's fields
- x and y have the same type

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

### 12.2. Generics in enums



```rust
enum Option<T> {
    Some(T),
    None,
}
```

### 12.3. Generics in methods

The generic parameter still has to be declared first: impl<T>. Only then can we use it in Point<T>, so that Rust knows the type in Point's angle brackets is generic rather than concrete. Note that Point<T> here is no longer a generic declaration but a complete struct type, because the struct we defined is Point<T>, not Point.

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

### 12.4. const generics (an important feature introduced in Rust 1.51)

This defines an array of type [T; N], where T is a type-based generic parameter and N is a value-based generic parameter, because it stands for the length of the array.  

N is a const generic, declared with the syntax const N: usize: a const generic N whose value is of type usize.  
T is bounded by std::fmt::Debug, which means T can be used in println!("{:?}", arr), because the {:?} formatting requires arr to implement that trait.  

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

#### 12.4.1 const generic expressions

Suppose some code has to run on a platform with very little memory, so the memory taken by function arguments must be limited. const generic expressions can express that:

```rust
// currently only available on nightly
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

fn something<T>(val: T)
where
    Assert<{ core::mem::size_of::<T>() < 768 }>: IsTrue,
    //       ^-----------------------------^ this is a const expression; any other const expression works too
{
    //
}

fn main() {
    something([0u8; 0]); // ok
    something([0u8; 512]); // ok
    something([0u8; 1024]); // compile error: the array is 1024 bytes, over the 768-byte argument limit
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

### 12.5. Performance of generics

Rust keeps generic code efficient by monomorphizing it at compile time. Monomorphization turns generic code into specific code by filling in the concrete types used when compiling.  

The compiler does the opposite of the steps we took to create the generic function: it looks at every place where generic code is called and generates code for the concrete types.  

An example using the standard library's Option enum:

```rust
let integer = Some(5);
let float = Some(5.0);
```

When Rust compiles this code, it monomorphizes it. The compiler reads the values passed to Option<T> and sees two kinds of Option<T>: one for i32 and one for f64. So it expands the generic definition Option<T> into Option_i32 and Option_f64, replacing the generic definition with these two specific ones.

The monomorphized code generated by the compiler looks like this:

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

### 12.6. Traits

If different types share the same behavior, we can define a trait and implement it for those types. A trait groups methods together to define a set of behaviors needed to achieve some purpose.  

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}
```

The trait keyword declares a trait, here named Summary. The curly brackets hold all methods of the trait, in this example: fn summarize(&self) -> String.

#### 12.6.1. Implementing a trait for a type

Implementing the Summary trait for Post and Weibo:

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}
pub struct Post {
    pub title: String, // title
    pub author: String, // author
    pub content: String, // content
}

impl Summary for Post {
    fn summarize(&self) -> String {
        format!("Article {}, author: {}", self.title, self.author)
    }
}

pub struct Weibo {
    pub username: String,
    pub content: String
}

impl Summary for Weibo {
    fn summarize(&self) -> String {
        format!("{} posted a weibo: {}", self.username, self.content)
    }
}

fn main() {
    let post = Post{title: "An introduction to Rust".to_string(),author: "Sunface".to_string(), content: "Rust is great!".to_string()};
    let weibo = Weibo{username: "sunface".to_string(),content: "Weibo seems less handy than Tweet".to_string()};

    println!("{}",post.summarize());
    println!("{}",weibo.summarize());
}
```

#### 12.6.2. Where traits are defined and implemented (the orphan rule)

There is one very important rule about where traits are implemented and defined: to implement trait T for type A, at least one of A or T must be defined in the current scope! For example, we can implement the standard library's Display trait for the Post type above, because Post is defined in the current scope. We can also implement Summary for String in the current crate, because Summary is defined in the current scope.

But you cannot implement Display for String in the current scope, because both are defined in the standard library, outside the current scope. This is called the orphan rule; it ensures that other people's code cannot break your code.

#### 12.6.3. Default implementations

A trait can define methods with a default implementation, so other types don't need to implement them, or can choose to override them:

```rust
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}
```

The code above defines a default implementation for Summary; let's write some code to test it:

```rust
impl Summary for Post {}

impl Summary for Weibo {
    fn summarize(&self) -> String {
        format!("{} posted a weibo: {}", self.username, self.content)
    }
}
```

Post uses the default implementation while Weibo overrides the method. The calls and output are:

```rust
    println!("{}",post.summarize());
    println!("{}",weibo.summarize());
(Read more...)
sunface posted a weibo: Weibo seems less handy than Tweet
```

#### 12.6.4. Traits as function parameters

```rust
pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
```

impl Summary means: an item parameter that implements the Summary trait.

Any type that implements Summary can be passed to this function, and the function body can call the trait's methods, such as summarize. Concretely, you can pass an instance of Post or Weibo, but other types such as String or i32 cannot be used, because they don't implement Summary.

#### 12.6.5. Trait bounds

The impl Trait syntax is just syntactic sugar:

```rust
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```

The full form is shown above; T: Summary is called a trait bound.

In simple cases the impl Trait sugar is enough, but in complex cases trait bounds give more flexibility and expressiveness. For example, a function taking two impl Summary parameters:

```rust
pub fn notify(item1: &impl Summary, item2: &impl Summary) {}
```

If the two parameters may have different types, the form above is fine, as long as both types implement Summary. But what if we want to force both parameters to have the same type? The syntax above can't express that; only a trait bound can:

```rust
pub fn notify<T: Summary>(item1: &T, item2: &T) {}
```

The generic type T says item1 and item2 must have the same type, and T: Summary says T must implement the Summary trait.

#### 12.6.6. Multiple bounds

Besides a single bound, we can specify several. For example, besides Summary, the parameter can also be required to implement Display, to control its formatted output:

```rust
pub fn notify(item: &(impl Summary + Display)) {}
```

Besides the sugared form above, the trait-bound form works too:

```rust
pub fn notify<T: Summary + Display>(item: &T) {}
```

With these two traits, we can call item.summarize and format item with println!("{}", item).

#### 12.6.7. where clauses

With many trait bounds, the function signature becomes hard to read:

```rust
fn some_function<T: Display + Clone, U: Clone + Debug>(t: &T, u: &U) -> i32 {}
```

A where clause improves its shape

```rust
fn some_function<T, U>(t: &T, u: &U) -> i32
    where T: Display + Clone,
          U: Clone + Debug
{}
```

#### 12.6.8. Conditionally implementing methods or traits with trait bounds

Trait bounds let us implement methods only for a given type combined with given traits, for example:

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

Not every Pair<T> has the cmp_display method: only a Pair<T> whose T implements both Display + PartialOrd does.  
This function is easier to read: the generic parameters, parameters and return value are together and quick to scan, while each generic parameter's traits are bounded on their own lines.  

Traits can also be implemented conditionally. For example, the standard library implements the ToString trait for any type that implements Display:

```rust
impl<T: Display> ToString for T {
    // --snip--
}
```

We can call the to_string method defined by ToString on any type that implements Display. For example, an integer can be turned into the corresponding String, because integers implement Display:

```rust
let s = 3.to_string();
```

#### 12.6.9. impl Trait in return position

impl Trait says that a function returns some type that implements a trait:

```rust
fn returns_summarizable() -> impl Summary {
    Weibo {
        username: String::from("sunface"),
        content: String::from("The M1 Max is amazing, the computer never lags anymore"),
    }
}
```

Returning impl Trait is extremely useful in one situation: when the real return type is very complex and you don't know how to write it (Rust requires every type to be spelled out), you can simply return impl Trait.  

This way of returning has one big limitation: there can only be one concrete type.  

#### 12.6.10. Implementing + for a custom type



```rust
use std::ops::Add;

// Derive Debug for the Point struct, for formatted output
#[derive(Debug)]
struct Point<T: Add<T, Output = T>> { // T must implement the Add trait, otherwise + is not possible.
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

### 12.7. Trait objects





## 13. Vectors

A vector can only store elements of the same type; to store elements of different types, use an enum or trait objects, as covered earlier.

### 13.1. Creating a vector

v is explicitly declared as Vec<i32>

> Vec::new

```rust
let v: Vec<i32> = Vec::new();
```

```rust
let mut v = Vec::new();
v.push(1);
```

Here v needs no type annotation, because from v.push(1) the compiler infers that its elements are i32, and therefore that v is Vec<i32>.

If you know in advance how many elements will be stored, create the vector with Vec::with_capacity(capacity). This avoids frequent allocations and copies when inserting lots of data, which improves performance.

> vec![]

The vec! macro creates a vector; unlike Vec::new, it can give initial values at creation:

```rust
let v = vec![1, 2, 3];
```

Again v needs no annotation: the compiler checks the elements and infers that v is Vec<i32> (integers default to i32 in Rust, as described under numeric types).

### 13.2. Updating a vector

To add an element at the end, use the push method:

```rust
let mut v = Vec::new();
v.push(1);
```

As with any other type, v must be declared mut before it can be modified.

### 13.3. Reading elements of a vector

There are two ways to read the element at a given position:

- Indexing.
- The get method.

```rust
let v = vec![1, 2, 3, 4, 5];

let third: &i32 = &v[2];
println!("The third element is {}", third);

match v.get(2) {
    Some(third) => println!("The third element is {third}"),
    None => println!("There is no third element at all!"),
}
```

> Indices of collection types start at 0: &v[2] borrows the third element of v and yields a reference to it. v.get(2) also accesses the third element, but returns Option<&T>, so an extra match is needed to get at the value.

> The difference between indexing and .get.

It matters when accessing past the end

```rust
let v = vec![1, 2, 3, 4, 5];

let does_not_exist = &v[100];
let does_not_exist = v.get(100);
```

Running the code above, the &v[100] access makes the program panic and exit, because it reads past the end of the vector. v.get does not: it handles this internally, returning Some(T) when there is a value and None when there isn't, so v.get is very safe to use.

### 13.4. Borrowing several vector elements at once

first = &v[0] is an immutable borrow and v.push is a mutable borrow. If first is not used after v.push, this code compiles, because of how the scope of a reference works.

```rust
let mut v = vec![1, 2, 3, 4, 5];

let first = &v[0];

v.push(6);

println!("The first element is: {first}");
//  ----- immutable borrow later used here
```

The reason: a vector's size can change. When the old buffer is too small, Rust allocates a bigger block of memory and copies the old vector into it. In that case the earlier reference would clearly point to invalid memory.

### 13.5. Iterating over the elements of a vector

Iterating over a vector is safer and more efficient than using indices (every index access triggers a bounds check):

```rust
let v = vec![1, 2, 3];
for i in &v {
    println!("{i}");
}
```

Elements of the vector can also be modified while iterating:

```rust
let mut v = vec![1, 2, 3];
for i in &mut v {
    *i += 10
}
```

### 13.6. Storing elements of different types

Enums and trait objects make it possible to store elements of different types.

> Enums

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

> Trait objects

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

In practice, vectors of trait objects are much more common than vectors of enums, mainly because trait objects are very flexible, while the compiler restricts enums more and doesn't allow adding types dynamically.

### 13.7. Common vector methods

> Ways to initialize a vec

```rust
fn main() {
    let v = vec![0; 3];   // default value 0, initial length 3
    let v_from = Vec::from([0, 0, 0]);
    assert_eq!(v, v_from);
}
```

A growable array means that when we add elements and capacity runs out, the vector grows (the current strategy allocates a block twice as large, copies all elements to it and updates the pointer). Clearly, frequent growth, or growth with many elements, means lots of memory copying that hurts performance.

Consider giving a realistic estimated capacity at initialization to minimize possible copying:

```rust
fn main() {
    let mut v = Vec::with_capacity(10);
    v.extend([1, 2, 3]);    // append data to v
    println!("Vector length: {}, capacity: {}", v.len(), v.capacity());

    v.reserve(100);        // adjust v's capacity to at least 100
    println!("Vector (reserve) length: {}, capacity: {}", v.len(), v.capacity());

    v.shrink_to_fit();     // release unused capacity; normally capacity is not released proactively
    println!("Vector (shrink_to_fit) length: {}, capacity: {}", v.len(), v.capacity());
}
```

### 13.8. Sorting vectors

There are two kinds of sort: the stable sort and sort_by, and the unstable sort_unstable and sort_unstable_by.

 Unstable doesn't mean the sorting algorithm itself is unreliable; it refers to how equal elements are treated. A stable sort never reorders equal elements; an unstable one doesn't guarantee that.

Overall, unstable sorting is faster than stable sorting, and a stable sort also allocates extra memory of half the size of the original array.

> Sorting an integer array

Here is an example of sorting a list of integers.

```rust
fn main() {
    let mut vec = vec![1, 5, 10, 2, 15];    
    vec.sort_unstable();    
    assert_eq!(vec, vec![1, 2, 5, 10, 15]);
}
```

> Sorting an array of floats

It turns out that floating-point numbers include a NAN value that can't be compared with other floats, so float types don't implement total ordering (Ord), only partial ordering (PartialOrd).

So if we are sure our float array contains no NAN values, we can use partial_cmp to compare.

```rust
fn main() {
    let mut vec = vec![1.0, 5.6, 10.3, 2.0, 15f32];    
    vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());    
    assert_eq!(vec, vec![1.0, 2.0, 5.6, 10.3, 15f32]);
}
```

> Sorting an array of structs

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
    // define a comparison that sorts by age in descending order
    people.sort_unstable_by(|a, b| b.age.cmp(&a.age));

    println!("{:?}", people);
}
```

## 14. HashMap

HashMap is another collection type in Rust's standard library. Unlike a vector, a HashMap stores one-to-one key-value (KV) pairs and offers lookups with an average complexity of O(1), which is very useful when you want to find a value by its key.

Rust's hash type (hash map) is HashMap<K,V>. Other languages have similar data structures: hash map, map, object, hash table, dictionary, and so on.

### 14.1. Creating a HashMap

As with creating a Vec, a HashMap can be created with new, and key-value pairs inserted with insert.

> Creating one with new

```rust
use std::collections::HashMap;

// Create a HashMap storing kinds of gems and their counts
let mut my_gems = HashMap::new();

// Write the gem kinds and their counts into the map
my_gems.insert("Ruby", 1);
my_gems.insert("Sapphire", 2);
my_gems.insert("Worthless rock from the riverbank mistaken for a gem", 18);
```

HashMap has to be brought into scope from the standard library with use .... Did we have to do that for the other two collection types, String and Vec? No, because HashMap is not in Rust's prelude (the most common types that Rust brings into scope automatically, for convenience).

All collection types are dynamic, meaning they have no fixed size in memory, so their data is stored on the heap and accessed through a reference stored on the stack. Like the other collections, HashMap is homogeneous: all keys must have the same type, and so must all values.

As with Vec, if you know in advance how many KV pairs will be stored, create a HashMap of that size with HashMap::with_capacity(capacity) to avoid frequent allocations and copies, which improves performance.

> Creating one with an iterator and collect

The into_iter method turns the list into an iterator, which collect then gathers. Note that collect can produce many kinds of target collections, so we need the type annotation HashMap<_,_> to tell the compiler: please collect into a HashMap

```rust
fn main() {
    use std::collections::HashMap;

    let teams_list = vec![
        ("Team China".to_string(), 100),
        ("Team USA".to_string(), 10),
        ("Team Japan".to_string(), 50),
    ];

    let teams_map: HashMap<_,_> = teams_list.into_iter().collect();
    
    println!("{:?}",teams_map)
}
```

### 14.2. Ownership transfer

- If a type implements the Copy trait, it is copied into the HashMap, so ownership doesn't matter
- If it doesn't implement Copy, ownership is moved into the HashMap

```rust
fn main() {
    use std::collections::HashMap;

    let name = String::from("Sunface");
    let age = 18;

    let mut handsome_boys = HashMap::new();
    handsome_boys.insert(name, age);

    println!("For being too shameless, {} has been removed from the list of handsome boys", name);
    println!("Also, his real age is way more than {}", age);
}
```

Running the code gives this error:

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
10 |     println!("For being too shameless, {} has been removed", name);
   |                                            ^^^^ value borrowed here after move
```

The hint is clear: name is a String, so it is subject to ownership rules. On insert, its ownership moved to handsome_boys, so using it at the end produces this merciless but expected error.

### 14.3. Querying a HashMap

The get method retrieves an element:

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();

scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);

let team_name = String::from("Blue");
let score: Option<&i32> = scores.get(&team_name);
```

- get returns an Option<&i32>: None when nothing is found, Some(&i32) when it is
- &i32 borrows the value in the HashMap; without borrowing, ownership might be moved

What if we want the score as a plain value? The answer is short but not simple:

```rust
let score: i32 = scores.get(&team_name).copied().unwrap_or(0);
```

### 14.4. Updating values in a HashMap

```rust
fn main() {
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    scores.insert("Blue", 10);

    // overwrite the existing value
    let old = scores.insert("Blue", 20);
    assert_eq!(old, Some(10));

    // look up the newly inserted value
    let new = scores.get("Blue");
    assert_eq!(new, Some(&20));

    // look up Yellow's value; insert a new value if it doesn't exist
    let v = scores.entry("Yellow").or_insert(5);
    assert_eq!(*v, 5); // didn't exist, so 5 was inserted

    // look up Yellow's value; insert a new value if it doesn't exist
    let v = scores.entry("Yellow").or_insert(50);
    assert_eq!(*v, 5); // already exists, so 50 was not inserted
}
```

A common pattern: look up a key's value, insert a new value if it doesn't exist, and update the existing value if it does. For example, counting word occurrences in a text:

```rust
use std::collections::HashMap;

let text = "hello world wonderful world";

let mut map = HashMap::new();
// split the string on spaces (English words are separated by spaces)
for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0);
    *count += 1;
}

println!("{:?}", map);
```

The code above creates a map that stores how often each word appears. Inserting a word checks: if it wasn't inserted before, use it as the key with 0 as the value; if it was, take the stored count and add one.

Two things are worth noting:

- or_insert returns a &mut v reference, so the corresponding value in the map can be modified directly through that mutable reference
- count must be dereferenced (*count) before use, otherwise the types don't match

### 14.5. Hash functions

```rust
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
// bring in a third-party hash function
use twox_hash::XxHash64;

// make the HashMap use the third-party hash function XxHash64
let mut hash: HashMap<_, _, BuildHasherDefault<XxHash64>> = Default::default();
hash.insert(42, "the answer");
assert_eq!(hash.get(&42), Some(&"the answer"));
```

HashMap currently uses the SipHash hash function. It is not very fast, but it is very secure. SipHash performs quite well on medium-sized keys, but not well enough for small keys (such as integers) or large keys (such as strings).

## 15. Lifetimes

### 15.1. Dangling pointers and lifetimes

The main purpose of lifetimes is to prevent dangling references, which would make a program reference data it shouldn't:

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

A few things are worth noting:

- The let r; declaration seems to risk using null, but in fact the compiler reports an error if we use it without initializing it
- r refers to the variable x inside the inner braces, but x is freed at the inner }, so back in the outer braces r would refer to an invalid x  

Here r is a dangling pointer, referring to the variable x that was freed too early. As expected, this code fails to compile:

```rust
error[E0597]: `x` does not live long enough // `x` doesn't live long enough
  --> src/main.rs:
   |
   |             r = &x;
   |                 ^^ borrowed value does not live long enough // the borrowed `x` doesn't live long enough
   |         }
   |         - `x` dropped here while still borrowed // `x` is dropped here, but it is still borrowed
   |
   |         println!("r: {}", r);
   |                           - borrow later used here // the borrow of `x` is used here
```

Here r has the larger scope; it lives longer. If Rust didn't prevent this dangling reference, then once x was freed, the value r refers to would no longer be valid, causing misbehavior in the program that can sometimes be very hard to find.

### 15.2. The borrow checker


### 15.3. Lifetimes in functions

The lifetime syntax is rather unusual: it starts with ' and the name is usually a single lowercase letter; most people use 'a. For a reference parameter, the lifetime goes after the & and is separated from the referenced type by a space:

```text
&i32        // a reference
&'a i32     // a reference with an explicit lifetime
&'a mut i32 // a mutable reference with an explicit lifetime
```

A single lifetime annotation doesn't mean much on its own, because lifetimes exist to tell the compiler how several references relate. For example, take a function whose first parameter first is a reference to an i32 with lifetime 'a, and whose second parameter second is also a reference to an i32 with lifetime 'a. The annotations only say that both first and second live at least as long as 'a; how long exactly, or which lives longer, is unknown:

## 16. Methods

### 16.1. Defining methods

Rust defines methods with impl, as in the following code:

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

In Rust, the definition of an object and of its methods are separate: struct Rectangle and impl Rectangle. Separating data from its use gives users great flexibility.  

impl Rectangle {} implements methods for Rectangle (impl is short for implementation); everything in the impl block is associated with Rectangle.  

### 16.2. self, &self and &mut self

In the signature of area we use &self instead of rectangle: &Rectangle; &self is short for self: &Self (note the case). Inside an impl block, Self refers to the struct type the methods are implemented for, and self refers to an instance of that type, here an instance of the Rectangle struct.  

Whichever struct the methods are implemented for, self refers to an instance of that struct.  

self still follows ownership:

- self means ownership of the Rectangle moves into the method; this form is rarely used
- &self means the method borrows the Rectangle immutably
- &mut self means a mutable borrow

We choose &self for the same reason we'd use &Rectangle in a function: we don't want ownership and don't need to change the value, only read the struct's data. To change the struct in the method, make the first parameter &mut self. Taking ownership of the instance with plain self as the first parameter is rare; it's usually used when the method turns the object into something else, after which the old object is no longer relevant, and it prevents accidental use of the old object.

### 16.3. Methods with the same name as a field

Rust allows a method to have the same name as a field of the struct:

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

When we write rect1.width(), Rust knows we're calling the method; rect1.width accesses the field.

Methods with the same name as a field are typically used as getters, for example:

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

This way, the fields of Rectangle can be private while its new and width methods are public: users can create a rectangle and get its width through the accessor rect1.width(), and since the width field is private, accessing rect1.width directly is an error. Note that in this example Self refers to Rectangle, the struct the methods are implemented for.  

### 16.4. Methods with more parameters

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

### 16.5. Associated functions

A function in an impl block without self is called an associated function: without self it can't be called as f.read(), so it is a function rather than a method, and because it lives in the impl block, closely tied to the struct, it's called an associated function.

```rust
impl Rectangle {
    fn new(w: u32, h: u32) -> Rectangle {
        Rectangle { width: w, height: h }
    }
}
```

Because it's a function, it can't be called with .; use :: instead, as in let sq = Rectangle::new(3, 3);. The function lives in the struct's namespace: the :: syntax is used for associated functions and for namespaces created by modules.

### 16.6. Multiple impl blocks

Rust allows several impl blocks for one struct, for flexibility and code organization. For example, when there are many methods, related ones can be grouped into their own impl block, giving several blocks that each serve one purpose:

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

### 16.7. Implementing methods on enums

Enums are powerful not only because they're convenient and can unify types, but also because, like structs, we can implement methods on them:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        // define the method body here
    }
}

fn main() {
    let m = Message::Write(String::from("hello"));
    m.call();
}
```

## 17. Creating macros with macro_rules!

Rust has a powerful macro system for metaprogramming. Macros don't produce function calls; they expand into source code that is compiled with the rest of the program. Unlike C and other languages, Rust macros expand into abstract syntax trees (AST) instead of being substituted as text by a preprocessor, so there are no unexpected precedence bugs.

Macros are created with the macro_rules! macro.

```rust
// This is a simple macro named `say_hello`.
macro_rules! say_hello {
    // `()` indicates that the macro takes no arguments.
    () => (
        // The macro will expand into the contents of this block.
        println!("Hello!");
    )
}

fn main() {
    // This call will expand into `println("Hello");`!
    say_hello!()
}
```

Why are macros useful?

- Don't repeat yourself (DRY). Often you need similar functionality for different types in several places; macros avoid the repetition.
- Domain-specific languages (DSL). Macros let you define special syntax for a specific purpose.
- Variadic interfaces. Interfaces that take a variable number of arguments, such as println!, which takes any number of arguments depending on the format string.

### 17.1. Designators

Macro arguments are prefixed with a dollar sign $ and annotated with a designator for their type (https://doc.rust-lang.org/reference/macros-by-example.html):

- block: a block expression
- expr: an expression
- ident: a variable or function name
- item Item
- literal: a literal constant
- pat: a pattern
- path: a TypePath-style path
- stmt: a statement
- tt: a token tree
- ty: a type
- vis: a visibility qualifier

```rust
macro_rules! print_result {
    // This macro takes an expression of type `expr` and prints it as a string along with its result.
    // The `expr` designator is used for expressions.
    ($expression:expr) => {
        // `stringify!` converts the expression *as it is* into a string.
        println!("{:?} = {:?}", stringify!($expression), $expression)
    };
}

fn main() {
    foo();
    bar();
    print_result!(1u32 + 1); // "1u32 + 1" = 2

    // Blocks are expressions too!
    print_result!({
        let x = 1u32;
        x * x + 2 * x - 1
    }); // "{ let x = 1u32; x * x + 2 * x - 1 }" = 2
}
```

### 17.2. Overloading

Macros can be overloaded to accept different combinations of arguments. In that sense, macro_rules! works like a match block:

```rust
macro_rules! test {
    // Arguments don't need to be separated by a comma. Any template can be used!
    ($left:expr; and $right:expr) => {
        println!(
            "{:?} and {:?} is {:?}",
            stringify!($left),
            stringify!($right),
            $left && $right
        )
    };
    // Each arm must end with a semicolon.
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

### 17.3. Repetition

In the argument list, a macro can use + to indicate that an argument may repeat at least once, and * to indicate that it may repeat zero or more times.

> Wrapping in $(...),+ matches one or more comma-separated expressions. Also note that the semicolon is optional on the last arm of a macro.

```rust
macro_rules! find_min {
    // Base case:
    ($x:expr) => ($x);
    // `$x` followed by at least one `$y,`
    ($x:expr, $($y:expr),+) => (
        // Call `find_min!` on the tail `$y`
        std::cmp::min($x, find_min!($($y),+))
    )
}

fn main() {
    println!("{}", find_min!(1u32)); // 1
    println!("{}", find_min!(1u32 + 2, 2u32)); // 2
    println!("{}", find_min!(5u32, 2u32 * 3, 4u32)); // 4
}
```

### 17.4. DRY (Don't Repeat Yourself)

By factoring out the common parts of functions or test suites, macros let you write DRY code. For example, implementing and testing the +=, *= and -= operators on Vec<T>:

```rust
use std::ops::{Add, Mul, Sub};

macro_rules! assert_equal_len {
    // The `tt` (token tree) designator is used for operators and tokens.
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
                // *x = x.$method(*y); // same effect as above
            }
        }
    };
}

// Implement the `add_assign`, `mul_assign` and `sub_assign` functions.
op!(add_assign, Add, +=, add);
op!(mul_assign, Mul, *=, mul);
op!(sub_assign, Sub, -=, sub);
```

### 17.5. DSL (domain-specific languages)

A DSL is a mini "language" embedded in a Rust macro. It is completely valid Rust, because the macro system expands it into normal Rust syntax trees; it just looks like another language. This lets you define concise, intuitive syntax for some special functionality (within limits).

Define a small calculator API: pass it an expression and it prints the result to the console.

```rust
macro_rules! calculate {
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force the type to integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};
}

fn main() {
    calculate! {
        eval 1 + 2 // hehehe `eval` is _not_ a Rust keyword!
    } // 1 + 2 = 3

    calculate! {
        eval (1 + 2) * (3 / 4)
    } // (1 + 2) * (3 / 4) = 0   ??
}
```

### 17.6. Variadic interfaces

A variadic interface takes an arbitrary number of arguments. println can, for example, with the number of arguments determined by the format string.

```rust
macro_rules! calculate {
    // The pattern for a single `eval`
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Force types to be integers
            println!("{} = {}", stringify!{$e}, val);
        }
    }};

    // Decompose multiple `eval`s recursively
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

fn main() {
    calculate! { // Look ma! Variadic `calculate!`!
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

## 18. Error handling

Error handling is the process of handling the possibility of failure. For example, failing to read a file and then continuing to use that bad input would clearly be problematic. Noticing and explicitly managing those errors saves the rest of the program from various pitfalls.

There are various ways to deal with errors in Rust:

- An explicit panic is mainly useful for tests and dealing with unrecoverable errors. For prototyping it can be useful, for example when dealing with functions that haven't been implemented yet, but in those cases the more descriptive unimplemented is better. In tests, panic is a reasonable way to explicitly fail.
- The Option type is for when a value is optional or when the lack of a value is not an error condition. For example, the parent of a directory: / and C: don't have one, which shouldn't be an error. When dealing with Options, unwrap is fine for prototyping and for cases where it's certain there is a value. However, expect is more useful, since it lets you specify an error message in case something goes wrong anyway.
- When there is a chance that things go wrong and the caller has to deal with the problem, use Result. You can unwrap and expect them too, but please don't do that unless it's a test or quick prototype.

### 18.1. panic

The simplest error handling mechanism is panic. It prints an error message, starts unwinding the stack, and usually exits the program. Here we explicitly call panic on our error condition:

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

### 18.2. Option and unwrap

The standard library (std) has an enum called Option<T> for cases where absence is a possibility. It manifests itself as one of two "options":

- Some(T): an element of type T was found
- None: no element was found

These cases can be handled explicitly with match, or implicitly with unwrap. Implicit handling either returns the inner element or panics.

The panic message can be customized manually with expect, but unwrap otherwise leaves us with a less meaningful output than explicit handling. In the following example, explicit handling yields a more controlled result while retaining the option to panic if desired.

```rust
// Handled explicitly with `match`.
fn give_commoner(gift: Option<&str>) {
    // Specify a course of action for each case.
    match gift {
        Some("snake") => println!("Yuck! I'm throwing that snake in a fire."),
        Some(inner) => println!("{}? How nice.", inner),
        None => println!("No gift? Oh well."),
    }
}

// Handled implicitly with `unwrap`.
fn give_princess(gift: Option<&str>) {
    // `unwrap` returns a `panic` when it receives a `None`.
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

### 18.3. Unpacking Options with ?

If x is an Option, evaluating x? returns the underlying value if x is Some; otherwise it terminates whatever function is executing and returns None.

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
    // Gets the area code of the phone number of the person's job, if it exists.
    fn work_phone_area_code(&self) -> Option<u8> {
        // This would need many nested `match` statements without the `?` operator.
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

### 18.4. Combinators: map

match is a valid way to handle Options, but using it heavily becomes tedious, especially with operations only valid for one kind of input. In those cases, combinators can manage control flow in a modular fashion.

Option has a built-in method map(), a combinator for the simple mapping of Some -> Some and None -> None. Several map() calls can be chained for even more flexibility.

In the following example, process() replaces all the functions before it while staying compact.

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

// Peeling food. If there isn't any, return `None`. Otherwise, return the peeled food.
fn peel(food: Option<Food>) -> Option<Peeled> {
    match food {
        Some(food) => Some(Peeled(food)),
        None => None,
    }
}

// Chopping food. If there isn't any, return `None`. Otherwise, return the chopped food.
fn chop(peeled: Option<Peeled>) -> Option<Chopped> {
    match peeled {
        Some(Peeled(food)) => Some(Chopped(food)),
        None => None,
    }
}

// Cooking food. Here, we use `map()` instead of `match` to handle the cases.
fn cook(chopped: Option<Chopped>) -> Option<Cooked> {
    chopped.map(|Chopped(food)| Cooked(food))
}

// A function to peel, chop, and cook food all in sequence. We chain multiple uses of `map()` to simplify the code.
fn process(food: Option<Food>) -> Option<Cooked> {
    food.map(|f| Peeled(f))
        .map(|Peeled(f)| Chopped(f))
        .map(|Chopped(f)| Cooked(f))
}

// Check whether there's food or not before trying to eat it!
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
    let cooked_carrot = process(carrot); // same result as above

    // Let's try the simpler looking `process()` now.
    let cooked_potato = process(potato);

    eat(cooked_apple); // Mmm. I love Cooked(Apple)
    eat(cooked_carrot); // Mmm. I love Cooked(carrot)
    eat(cooked_potato); // Oh no! It wasn't edible.
}
```

### 18.5. Combinators: and_then

map() simplifies match statements with chained calls. However, using map() with a function that returns an Option<T> results in the nested Option<Option<T>>, and chaining multiple calls together becomes confusing. That's where and_then() comes in, known in some languages as flatmap.

and_then() calls its function input with the wrapped value and returns the result. If the Option is None, it returns None instead.

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

// We don't have the ingredients to make Sushi (we have others).
fn have_ingredients(food: Food) -> Option<Food> {
    match food {
        Food::Sushi => None,
        _ => Some(food),
    }
}

// We have the recipe for everything except Cordon Bleu.
fn have_recipe(food: Food) -> Option<Food> {
    match food {
        Food::CordonBleu => None,
        _ => Some(food),
    }
}

// A series of `match`es expressing this logic:
fn cookable_v1(food: Food) -> Option<Food> {
    match have_ingredients(food) {
        None => None,
        Some(food) => match have_recipe(food) {
            None => None,
            Some(food) => Some(food),
        },
    }
}

// The logic above can be rewritten more compactly with `and_then()`:
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

### 18.6. Result

Result is a richer version of the Option type that describes a possible error instead of a possible absence.

That is, Result<T, E> can have one of two outcomes:

- Ok<T>: an element T was found
- Err<E>: an error was found with element E, the type of the error.

By convention, the expected outcome is Ok, while the unexpected outcome is Err.

Like Option, Result has many associated methods. unwrap(), for example, either yields the element T or panics. For case handling, there are many combinators shared between Result and Option.

Working with Rust, you'll likely meet methods that return a Result, such as parse(). It can't always parse a string into the requested type, so parse() returns a Result indicating possible failure.

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

In the failing case, parse() leaves us with an error for unwrap() to panic on. The panic also exits our program and gives an unpleasant error message.

To improve the quality of the error message, we should be more specific about the return type and consider explicitly handling the error.

### 18.7. map for Result

In general, we want to return the error to the caller so it can decide the right way to respond.

First we need to know what kind of error type we're dealing with. To find the Err type, we look at parse(), which is implemented with the FromStr trait for i32. As a result, the Err type is specified as ParseIntError.

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
    // This still presents a reasonable answer.
    let twenty = multiply("10", "2");
    print(twenty); // n is 20

    // The following now provides a much more helpful error message.
    let tt = multiply("t", "2");
    print(tt); // Error: invalid digit found in string
}
```

### 18.8. Aliases for Result

Aliases are especially helpful at the module level. Errors in one module often share the same Err type, so a single alias can succinctly define all associated Results. The standard library even provides one: io::Result!

```rust
use std::num::ParseIntError;

// Define a generic alias for a `Result` with the error type `ParseIntError`.
type AliasedResult<T> = Result<T, ParseIntError>;

// Use the alias above to refer to our specific `Result<i32,ParseIntError>` type from the previous section.
fn multiply(first_number_str: &str, second_number_str: &str) -> AliasedResult<i32> {
    first_number_str.parse::<i32>().and_then(|first_number| {
        second_number_str
            .parse::<i32>()
            .map(|second_number| first_number * second_number)
    })
}

// Here, the alias again saves us some code.
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

### 18.9. Early returns

Another way to deal with errors is to combine match statements with early returns.

If an error occurs, we can stop executing the function and return the error. For some, this style of code is easier to both read and write. Here is the earlier example rewritten with early returns:

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

### 18.10. Introducing ?

Sometimes we just want the simplicity of unwrap without the possibility of a panic. Until now, handling unwrap errors forced us to nest deeper and deeper when all we wanted was to get the variable out. ? is exactly for this: ? is almost exactly equivalent to an unwrap that returns instead of panicking on Err.

Upon finding an Err, there are two valid actions:

- panic!, which we already decided to avoid where possible.
- return it, because an Err means it cannot be handled.

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

### 18.11. The try! macro

Before ? existed, the same functionality was achieved with the try! macro. The ? operator is now recommended, but you may still find try! in older code. With try!, the multiply function from the previous example would look like this:

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

### 18.12. Handling multiple error types

Sometimes an Option needs to interact with a Result, or a Result<T, Error1> with a Result<T, Error2>. In those cases we want to manage the different error types in a way that makes them composable and easy to interact with.

In the following code, two instances of unwrap generate different error types. Vec::first returns an Option, while parse::<i32> returns a Result<i32, ParseIntError>:

```rust
fn double_first(vec: Vec<&str>) -> i32 {
    let first = vec.first().unwrap(); // Generate error 1
    2 * first.parse::<i32>().unwrap() // Generate error 2
}

fn main() {
    let numbers = vec!["42", "93", "18"];
    let empty = vec![];
    let strings = vec!["tofu", "93", "18"];

    println!("The first doubled is {}", double_first(numbers)); // The first doubled is 84
 
    println!("The first doubled is {}", double_first(empty)); // called `Option::unwrap()` on a `None` value
    // Error 1: the input vector is empty

    println!("The first doubled is {}", double_first(strings)); // called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
    // Error 2: the element doesn't parse to a number
}
```

### 18.13. Pulling Results out of Options

The most basic way of handling mixed error types is to embed them in each other. If the Option is None, error handling continues. Some combinators make it easy to swap a Result and an Option.

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

### 18.14. Defining an error type

Rust allows us to define our own error types. In general, a "good" error type:

- represents different errors with the same type
- presents nice error messages to the user
- is easy to compare with other types
  - Good: Err(EmptyVec)
  - Bad: Err("Please use a vector with at least one element".to_owned())
- can hold information about the error
  - Good: Err(BadChar(c, position))
  - Bad: Err("+ cannot be used here".to_owned())
- composes well with other errors

```rust
use std::error;
use std::fmt;

type Result<T> = std::result::Result<T, DoubleError>;

#[derive(Debug, Clone)]
// Define our error type. It can be customized for our error handling cases.
// We can write our own errors, defer to an underlying error implementation, or do something in between.
struct DoubleError;

// It stores no extra information about the error, so without changing our error type we can't say which string failed to parse.
impl fmt::Display for DoubleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// Implement the `Error` trait for `DoubleError`, so other errors can wrap this error type.
impl error::Error for DoubleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None // Generic error, doesn't track the internal cause.
    }
}

fn double_first(vec: Vec<&str>) -> Result<i32> {
    vec.first()
        .ok_or(DoubleError) // Change the error to our new type.
        .and_then(|s| {
            s.parse::<i32>()
                .map_err(|_| DoubleError) // Update to the new error type here too.
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

### 18.15. Boxing errors

One way to write simple code while preserving the original errors is to Box them. The drawback is that the underlying error type is only known at runtime, not statically determined.

### 18.16. Other uses of ?

? actually means either unwrap or return Err(From::from(err)). Since From::from converts between different types, if you use ? where the error can be converted to the return type, it converts automatically.

```rust
// Use `?` to get the inner value immediately.
fn double_first(vec: Vec<&str>) -> Result<i32> {
    let first = vec.first().ok_or(EmptyVec)?;
    let parsed = first.parse::<i32>()?;
    Ok(2 * parsed)
}
```

### 18.17. Wrapping errors


### 18.18. Iterating over Results

An Iter::map operation might fail


#### 18.18.1. Ignoring failed items with filter_map()

filter_map calls a function and filters out the results that are None.

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

#### 18.18.2. Failing the entire operation with collect()

Result implements FromIter, so a vector of results (Vec<Result<T, E>>) can be turned into a result wrapping a vector (Result<Vec<T>, E>). Once a Result::Err is found, the iteration stops.

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

#### 18.18.3. Collecting all valid values and errors with partition()

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












## x. Modules

Rust has many features for managing the organization of your code, including which details are exposed and which are private, and which names are in each scope of the program. Together they are called "the module system":

- Packages: a Cargo feature that lets you build, test and share crates.
- Crates: a tree of modules that produces a library or executable.
- Modules and use: let you control the organization, scope and privacy of paths.
- Paths: a way of naming an item, such as a struct, function or module

### x.1. Packages and crates

A crate is a binary or a library. The crate root is a source file that the Rust compiler starts from and that makes up the root module of your crate.  
A package is one or more crates that provide a set of functionality. A package contains a Cargo.toml file that describes how to build those crates.  
A package can contain at most one library crate and any number of binary crates, but it must contain at least one crate (library or binary).  

The cargo new command

```text
$ cargo new my-project
     Created binary (application) `my-project` package
$ ls my-project
Cargo.toml
src
$ ls my-project/src
main.rs
```

### x.2 Defining modules to control scope and privacy

A front_of_house module containing other modules that contain functions

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

The structure of the module tree.

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

A path takes two forms:

- An absolute path starts from the crate root, beginning with the crate name or the literal crate.
- A relative path starts from the current module and uses self, super or an identifier in the current module.

The following code fails to compile; ignore that for now

```rs
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // absolute path
    crate::front_of_house::hosting::add_to_waitlist();

    // relative path
    front_of_house::hosting::add_to_waitlist();
}
```

> Exposing paths with the pub keyword

```rs
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub fn eat_at_restaurant() {
    // absolute path
    crate::front_of_house::hosting::add_to_waitlist();

    // relative path
    front_of_house::hosting::add_to_waitlist();
}
```

> Relative paths starting with super

Use super to build a relative path that starts from the parent module. This is like starting a filesystem path with ..

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

> Bringing paths into scope with the use keyword

Adding use and a path in a scope is similar to creating a symbolic link in the filesystem. By adding use crate::front_of_house::hosting in the crate root, hosting is now a valid name in that scope, just as if the hosting module had been defined in the crate root. Paths brought into scope with use also check privacy, like any other paths.

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

> Providing new names with the as keyword

Use as to give a new local name, or alias

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

### x.3 Library packages

```rust
$ cargo new my-lib --lib
     Created library `my-lib` package
$ ls my-lib
Cargo.toml
src
$ ls my-lib/src
lib.rs
```

#### x.3.1. Package layout

A typical package in a real project contains several binary crates, placed in the src/bin directory with each file a separate binary crate, plus a library crate, of which there can only be one: src/lib.rs:

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

- The only library crate: src/lib.rs
- The default binary crate: src/main.rs; the compiled executable has the same name as the package
- Other binary crates: src/bin/main1.rs and src/bin/main2.rs, each producing an executable with the same name as its file
- Integration tests: in the tests directory
- Benchmarks: in the benches directory
- Examples: in the examples directory







