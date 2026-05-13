# trash-cli
**仅适用于 macOS（已在 macOS Tahoe 26 测试通过）**

`walavave/trash-cli` 是一款专为 macOS 回收站设计的 Rust 命令行工具。
它实现了基于文件系统的文件删除、列出、恢复、清空和选择性删除工作流，
**无需依赖 Finder 或 AppleScript** 即可操作原生 macOS 回收站。

## 核心特性
- 支持完整命令：`put`、`list`、`restore`、`empty`、`rm`
- 直接读取 macOS 原生回收站元数据（`.DS_Store`）
- 写入新的删除记录到 macOS 原生回收站结构
- 兼容用户主目录回收站、磁盘根目录回收站、挂载卷回收站和自定义回收站根目录
- 支持恢复时覆盖控制、交互式多选恢复

## 安装方式

**推荐**：下载预编译二进制文件，解压并移动到系统可执行路径（**仅支持 ARM 架构 Mac**）：
```sh
tar -xzf "trash-cli-${VERSION}-aarch64-apple-darwin.tar.gz"
sudo install -m 755 "trash-cli-${VERSION}-aarch64-apple-darwin/trash" /usr/local/bin/trash
```

第一次运行命令，如`trash --version`会弹出警告，进入系统设置>隐私与安全性。最下面点击`仍要打开`。

或通过 Cargo 从源码编译安装：
```sh
cargo install --path . --locked
```

macOS 用户也可以使用 Homebrew 安装：
```sh
brew tap walavave/tap
brew install --formula walavave/tap/trash-cli
```

## 使用说明
### 命令总览
```text
trash list [--sort date|path|none] [--trash-dir DIR] [PATH]
trash restore [--sort date|path|none] [--trash-dir DIR] [--overwrite] [PATH]
trash put [--trash-dir DIR] FILE...
trash empty [--trash-dir DIR] [DAYS]
trash rm [--trash-dir DIR] PATTERN
```

未输入命令时，工具会自动显示帮助信息。

**全局选项**：
- `-h`, `--help`：显示帮助文档
- `--version`：显示版本信息

---

### `rm` 匹配规则
- 如果模式以 `/` 开头，匹配**完整原始路径**
- 否则，仅匹配**文件/目录名称**
- 支持通配符：`*`（匹配任意字符）、`?`（匹配单个字符）
- 使用时请给模式加引号，避免被 Shell 提前解析

### 命令详解
#### `put`
将文件或目录移动到回收站。
```sh
trash put ./foo.txt ./build.log
trash put ./dir-a ./dir-b
```
- 若目标回收站存在同名文件，会自动重命名为 `name_1`、`name_2` 等
- 该命令会自动更新 `.DS_Store` 元数据

#### `list`
列出回收站中的文件。
```sh
trash list
trash list ./src
trash list --sort path
```

输出格式：
```text
YYYY-MM-DD HH:MM:SS /original/path
```
- 指定 `PATH` 时，仅显示原始路径位于该目录下的文件
- **显示的时间为文件最后修改时间，并非删除时间**

#### `restore`
交互式恢复回收站文件。
```sh
trash restore
trash restore ./src
trash restore --overwrite ./src
```
- 匹配项会显示从 0 开始的序号
- 支持输入：单个序号、逗号分隔序号、范围（如 `0,2-4`）
- 直接回车不选择，不恢复任何文件

#### `empty`
**永久删除**回收站文件。
```sh
trash empty
trash empty 7
```
- 不指定天数：清空所有文件
- 指定天数：仅删除超过该天数的文件

#### `rm`
根据匹配规则**永久删除**指定回收站文件。
```sh
trash rm "*.o"
trash rm "/workspace/tmp/*"
```

## 编译构建
基础编译：
```sh
cargo build
```

构建发布版可执行文件（生成 `trash`）：
```sh
cargo build --release
./target/release/trash --version
```

运行测试：
```sh
cargo test
```

## 相关项目

如果你使用 Linux，推荐使用[trash-cli](https://github.com/andreafrancia/trash-cli)