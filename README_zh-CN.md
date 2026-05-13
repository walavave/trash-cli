# trash-cli-macos

`trash-cli-macos` 是一个面向 macOS 回收站目录的 Rust 命令行工具。
它通过纯文件系统方式实现放入回收站、列出、恢复、清空和按模式删除，
不依赖 Finder 或 AppleScript。

英文版文档见 [README.md](README.md)。

## 功能

- 支持与 `trash-cli` 常见用法对应的命令集合：
  `put`、`list`、`restore`、`empty` 和 `rm`
- 能从 `.DS_Store` 读取 macOS 原生回收站元数据
- 会把新放入回收站的条目直接写回 macOS 原生 Trash 结构
- 支持 home 回收站、卷顶层回收站、挂载卷回收站和自定义回收站根目录
- 支持恢复时覆盖控制和交互式多选恢复

## 当前实现状态

这个项目刻意不调用 Finder API，所有操作都直接针对 macOS 原生 Trash：

- 被回收的文件直接移动到原生 Trash 目录
- 原始路径等元数据写入 `.DS_Store`

这样实现的目的，是在不依赖 Finder 的前提下，仍然保持与 macOS Trash
目录结构一致，而不是再引入一层自定义侧边目录。

## 支持的目录

- `~/.Trash`
- `/.Trashes/<uid>`
- `/Volumes/*/.Trashes/<uid>`
- 自定义 `--trash-dir DIR` 根目录

自定义根目录既可以包含：

- 直接存放在根目录中的原生 macOS Trash 条目
- 通过 `.DS_Store` 表示的原生 macOS 元数据

## 构建

```sh
cargo build
```

构建安装后的实际命令 `trash`：

```sh
cargo build --release
./target/release/trash --version
```

运行测试：

```sh
cargo test
```

## Homebrew 安装

```sh
brew tap walavave/tap
brew install --formula walavave/tap/trash-cli
```

## 命令概览

当前二进制是单个可执行文件，通过子命令工作：

```text
trash restore [OPTIONS] [PATH]
trash list [OPTIONS] [PATH]
trash put [OPTIONS] FILE...
trash empty [OPTIONS] [DAYS]
trash rm [OPTIONS] PATTERN
```

如果不显式传入命令，会直接显示帮助。

全局选项：

- `--trash-dir DIR` 扫描或操作指定回收站根目录
- `--sort date|path|none` 为 `list` 和 `restore` 候选项排序
- `--overwrite` 允许 `restore` 覆盖目标位置已有文件
- `-h`、`--help` 显示帮助
- `--version` 显示版本

## 命令详情

### `put`

把一个或多个文件或目录移动到回收站。

```sh
trash put ./foo.txt ./build.log
trash put ./dir-a ./dir-b
```

说明：

- 如果目标回收站里已经存在同名条目，会自动生成唯一名称，例如 `name_1`、`name_2`
- 该命令会更新原生 `.DS_Store` 元数据

### `list`

列出回收站文件。

```sh
trash list
trash list ./src
trash list --sort path
```

输出格式：

```text
YYYY-MM-DD HH:MM:SS /original/path
```

如果提供 `PATH`，只显示原始路径与该路径相同或位于其内部的条目。

### `restore`

交互式恢复回收站文件。

```sh
trash restore
trash restore ./src
trash restore --overwrite ./src
```

行为：

- 会先显示匹配条目及其从 0 开始的索引
- 输入支持单个索引、逗号分隔索引和范围，如 `0,2-4`
- 直接按回车表示不恢复任何文件

### `empty`

永久删除回收站条目。

```sh
trash empty
trash empty 7
```

行为：

- 不带 `DAYS` 时，会删除扫描到的全部回收站条目
- 带 `DAYS` 时，只删除至少在这么多天之前进入回收站的条目

### `rm`

永久删除与模式匹配的回收站条目。

```sh
trash rm '*.o'
trash rm '/workspace/tmp/*'
```

行为：

- 如果模式以 `/` 开头，则匹配完整原始路径
- 否则只匹配 basename
- 支持的通配符只有 `*` 和 `?`
- 传入模式时应加引号，避免被 shell 提前展开

## 与上游 `trash-cli` 的差异

- 这是一个针对 macOS 回收站目录的 Rust 实现
- 当前是单个二进制加子命令，而不是多个独立安装的命令
- 不依赖 Finder 集成
- 通过 `.DS_Store` 读取和写入原生 macOS 回收站元数据

## 注意事项

- 原生 macOS 条目依赖 `.DS_Store` 可读
- 如果原生回收站目录里还有文件，但缺少对应元数据，扫描时会给出 warning 并跳过
- 对原生条目，当前显示时间直接来自垃圾桶中文件的修改时间，不是专门的删除时间
- `restore` 默认拒绝覆盖已有目标，只有显式传入 `--overwrite` 才会覆盖

## 示例流程

```sh
trash put ./notes.txt ./tmp/output.log
trash list
trash rm '*.log'
trash restore ./notes.txt
trash empty 30
```
