# 开发进度与 AI 续接

更新日期：2026-11-05（Asia/Shanghai）。面向没有聊天上下文的 AI：先读本文件，再查看实际源码。本文替代 2026-11-04 的旧快照。

## 先读结论

- 用户通过 Windows Rust CLI 项目学习完整开发流程，生产实现必须由用户自己写。
- CLI 参数解析已完成当前阶段，之前验证过 25 个测试通过。
- 普通文件、空目录与递归目录删除已经实现，本次验证 6 个删除测试通过。
- 当前停在预览功能的 TDD 红灯阶段：AI 按用户明确要求写好了预览测试，用户正在写实现。不要把预期红灯当作回归，也不要自动把生产代码补完。
- 用户已完成 preview_path 的外层结果收集，collect_preview 仍是返回 Ok(()) 的空实现，所以所有预览结果目前都是空列表。
- main 仍仅输出 Hello, world!。CLI 未接入实际删除或预览，尚不能作为完整命令行工具使用。

## 仓库与交接分支

- 远程：https://github.com/zu0my/tools.git
- 本次交接分支：codex/remove-preview-handoff。
- 分支起点：main 上的 475588f（初始化 Rust CLI 项目并实现删除参数解析）。后续删除实现、预览测试和本文在交接分支保存。
- 换电脑先 fetch 并切换到 origin/codex/remove-preview-handoff 对应的本地跟踪分支；不要只停留在 main。
- 本地目录可以不同，所有文档使用仓库相对路径。不要同步 target 构建产物。
- Git 同步整合使用 rebase，不使用 merge；保留用户未提交修改。

## 用户希望怎样协作

用户了解部分语法和生命周期，但不熟悉标准库、第三方 crate、项目文件与模块组织。

1. 生产实现由用户写。AI 提供需求澄清、库与 API 解释、文件位置、模块职责、伪代码和审查，不生成或自动修改生产实现。
2. 用户明确授权 AI 直接编写了本轮预览测试；已有测试可以在这次授权范围内维护。这不授权 AI 自动实现 collect_preview。后续扩大代写范围以用户指令为准。
3. 使用 TDD：红、绿、重构。一轮按完整行为规则组织，不再把每个选项、一个机械判断拆成许多聊天回合。
4. 每轮给出涉及文件、所需 API、测试名称、输入、预期、伪代码和精确的局部测试命令。已有测试可直接运行，不必重新口述让用户重写。
5. 请求审查时先读取当前代码；没有待修正问题时可直接开始下一轮指导，不需要等待“继续”。临时只问语法或交接文档时，不擅自推进实现。
6. 不做全量测试；按更改范围测试。不能把运行 0 个测试、空测试体或预期失败说成通过。
7. Windows 命令通过 pwsh.exe（PowerShell 7）执行。缺少 Rust 等环境时提示用户决定是否安装，不自行下载。
8. 用户有意保留 ..RemoveOptions::default()，便于未来字段自动补齐，不要反复要求删除。重点测试字段可显式指定。

详细共同约定见 [学习与协作](LEARNING.md)、[开发约定](DEVELOPMENT.md)。

## 已确定的产品规则

形式：tools remove [选项] <路径>...。tools 暂用包名。

- 默认永久删除，不进回收站，不弹确认。
- 普通文件和空目录可直接删除；非空目录必须 -r 或 --recursive。
- --dry-run 返回并最终展示拟删除路径，不修改文件系统，不读取文件正文。
- 选项在路径前。遇到首个路径或 -- 后，所有后续参数都是路径，包括 -r、--help、--。
- 支持多个明确目标，计划逐个处理，一个失败继续其他目标，不回滚已删除内容。多目标执行尚未接入。
- 未知命令、未知选项、缺少路径是解析错误；不存在路径或系统拒绝是执行错误。
- 计划退出码：0 成功，1 存在执行失败，2 参数错误；main 尚未实现映射。
- 不提供 --force 或工具自身的通配符展开。
- 链接应只处理链接本身，不遍历目标。Windows 目录链接真实删除和特殊重解析点支持尚未验证完成，不要声称已支持。

完整契约见 [删除工具](tools/remove.md)。

## 实际代码与文件职责

| 文件 | 当前内容 |
| --- | --- |
| src/main.rs | 声明 cli、remove 模块；入口只打印 Hello, world! |
| src/cli.rs | parse_args、parse_remove_args，接收 &[OsString]；25 个测试；无文件系统副作用 |
| src/remove.rs | remove_path 实际删除；preview_path 外层收集；collect_preview 占位；6 个删除测试与7个预览测试 |
| Cargo.toml | 单 Cargo 包，Rust 2024 edition；普通依赖为空；tempfile = "3.27.0" 是开发依赖 |
| Cargo.lock | 已纳入仓库，换电脑一起同步 |

不要提前加 lib.rs、workspace、多层 services 目录、通用 trait 或插件框架。当前模块内拆函数足够。

CLI 类型：Command 为 Remove(RemoveOptions)、Help、RemoveHelp；RemoveOptions 保存 Vec<PathBuf>、recursive、dry_run，并派生 Default；ParseError 为 MissingPath、UnknownCommand(OsString)、UnknownOption(OsString)。

删除接口：remove_path(&Path, recursive: bool) -> io::Result<()>。读取 symlink_metadata，普通目录按 recursive 选择 remove_dir_all 或 remove_dir，其余走 remove_file。当前没有 dry_run 参数；预览另用独立函数，不能恢复成忽略删除的空开关。

预览接口：

- preview_path(&Path, recursive: bool) -> io::Result<Vec<PathBuf>>：当前创建结果 Vec，调用 collect_preview 并用 ? 传播错误，最后返回 Vec。这部分用户已写。
- collect_preview(&Path, recursive: bool, &mut Vec<PathBuf>) -> io::Result<()>：当前函数体仅 Ok(())，下一步由用户实现。

## 本轮预览结果与展开规则

| 目标 | recursive | 行为 |
| --- | --- | --- |
| 普通文件 | 任意 | 结果仅包含自身路径 |
| 空目录 | 任意 | 结果仅包含自身路径 |
| 非空目录 | false | DirectoryNotEmpty 错误 |
| 非空目录 | true | 根目录与全部后代，按下述确定顺序展开 |
| 符号链接 | 任意 | 只列链接自身，不遍历目标 |
| 不存在或无法读取 | 任意 | 返回对应错误 |
| 其他不支持的类型 | 任意 | 返回 Unsupported；Windows 特殊重解析点另行验证 |

顺序采用先序、深度优先：当前目录在前，同层子项按 PathBuf 排序后，逐个递归展开。不是把最终列表整体排序。文件与目录都要列出。

示例树：

~~~text
target/
  a.txt
  nested/
    b.txt
~~~

结果依次是 target、target/a.txt、target/nested、target/nested/b.txt（实际为系统 PathBuf）。相对路径保持相对路径，不 canonicalize，不解析到链接目标。

预览不打印、不读文件正文、不创建或删除文件，不为探测可删除性尝试真实删除。能展开不意味着未来一定删得掉。单目标展开任一步失败则返回 Err，不返回成功的部分列表；内部 Vec 可能已部分填入，由外层丢弃即可。

## 用户下一步需要实现的伪代码

~~~text
collect_preview：
    用 symlink_metadata 读取当前路径自身元数据，错误向上传播
    如果是符号链接或普通文件：
        将当前路径的拥有副本加入结果
        返回成功
    如果不是普通目录：
        返回 Unsupported

    read_dir 读取所有直接子项：
        每个迭代结果都可能失败，要处理错误
        将 entry.path() 加入 children

    如果没有递归且 children 非空：
        返回 DirectoryNotEmpty

    将当前目录加入结果
    如果递归：
        对 children 排序
        按顺序递归调用 collect_preview，错误向上传播
    返回成功
~~~

需要讲解而不是代写的 API：Metadata::file_type、FileType::is_symlink/is_file/is_dir、read_dir、DirEntry::path、Path::to_path_buf、Vec::sort、io::Error::new、ErrorKind::DirectoryNotEmpty/Unsupported、? 和 &mut Vec<PathBuf>。

read_dir 的外层调用和每个子项都是可失败操作；不要 flatten 或 filter_map(Result::ok) 静默吞错。读目录顺序不保证稳定，必须显式排序。FileType 判断配合 symlink_metadata，避免跟随链接。

参考官方资料：[read_dir](https://doc.rust-lang.org/std/fs/fn.read_dir.html)、[FileType](https://doc.rust-lang.org/std/fs/struct.FileType.html)、[remove_dir_all](https://doc.rust-lang.org/std/fs/fn.remove_dir_all.html)。

## 现有测试与本次验证

本次交接前实际运行：

~~~text
cargo test remove::tests::
~~~

结果：6 passed、6 failed、1 ignored、25 filtered out，退出码 1。这是当前预期的 TDD 红灯，不是全部通过；未运行全量测试。

删除测试全部通过：

- remove_existing_file_deletes_it
- remove_missing_file_returns_not_found
- remove_empty_directory_deletes_it
- remove_nonempty_directory_returns_error_and_preserves_contents
- remove_nonempty_directory_recursively_deletes_tree（完整嵌套树，只删除根目标一次）
- remove_file_with_recursive_enabled_deletes_it

预览测试由 AI 写好，6 个均因空实现返回空列表而失败：

| 测试名称 | 核心断言 |
| --- | --- |
| preview_file_returns_path_without_deleting_it | false/true 都只返回文件自身，文件仍存在 |
| preview_empty_directory_returns_directory_without_deleting_it | false/true 都返回空目录自身，目录仍存在 |
| preview_nonempty_directory_without_recursion_returns_error | DirectoryNotEmpty，目录和文件均保留 |
| preview_recursive_directory_returns_all_paths_in_order | 完整嵌套路径和先序顺序，所有对象仍存在 |
| preview_directory_sorts_siblings | 故意先创建 z.txt 后创建 a.txt，结果仍是目录、a、z |
| preview_missing_path_returns_not_found | false/true 都返回 NotFound，不创建路径 |

另有 Windows 测试 preview_symlink_does_not_expand_target，带显式 #[ignore] 原因：当前环境 symlink_dir 返回 Windows 错误 1314“客户端没有所需的特权”。需要开发者模式或创建符号链接权限。已实际尝试过，失败在测试准备阶段，不是预览断言。不要无声跳过后声称链接功能已验证，也不要擅自更改系统配置。

在具备权限的环境手动运行：

~~~text
cargo test remove::tests::preview_symlink_does_not_expand_target -- --ignored --exact
~~~

测试只验证目录符号链接本身不被展开，不能证明链接位于目录树中、断链或 junction 等场景全部已覆盖。collect_preview 的三个未使用参数警告来自占位实现，正常实现后再看是否仍有警告，不急于统一加下划线或屏蔽。

CLI 的 25 个测试在前一阶段实际通过，源码此后未改，本次没有重新运行。不能把 filtered out 说成本次通过。

## 续接时的操作

1. 确认在交接分支，读取实际 src/remove.rs，保留任何用户新增实现。
2. 若用户未有新问题，直接从 collect_preview 的实现指导开始；preview_path 外层已完成，不重复让用户重写。
3. 不自动修复所有红灯，不修改测试预期以迁就空实现，不加入真正删除来模拟预览。
4. 局部验证可用 cargo test remove::tests::preview_；实现完成时预期 6 passed、0 failed、1 ignored。
5. 需要验证本模块回归时运行 cargo test remove::tests::；完成后预期 12 passed、0 failed、1 ignored。符号链接测试另行执行。
6. 预览完成并审查通过后，再与用户推进 CLI 调度、多目标处理、输出与退出码；当前不抢先实现这些内容。

## 已学到、不要重复踩的点

- tempfile::tempdir() 创建独立目录并由 TempDir 自动清理；tempfile::env::temp_dir() 仅返回共享临时目录路径，两者不同。
- 路径 join("nonempty") 才是在临时目录下创建子项；join("/nonempty") 在 Windows 会替换盘符后的目录部分，可能落到盘符根目录。
- TempDir 一直保留到测试结束，测试目标创建为其下的子目录；不要清理任意根目录或共享临时路径。
- 用户之前可能运行旧测试留下根目录 nonempty 或共享临时文件，但具体残留与内容没有确认。本次没有检查或删除，接手 AI 不得猜测后自动清理。
- 测试拒绝删除非空目录时传目录路径，不要误传内部文件路径。
- 检查文件是否仍存在使用 try_exists，检查错误用 expect，断言消息表达状态不符合预期，两者不能混用。
- 验证嵌套递归时不能先手动删除子目录；一次调用根目标才能测试完整展开/删除。
- Result 尾表达式与 return 的区别、切片借用、OsString 到 PathBuf 等已讨论，按当前问题解释即可，不重新从语法课讲起。

## 可直接给下一位 AI 的开场说明

> 请先读 docs/PROGRESS.md、docs/LEARNING.md 和 src/remove.rs。当前在 codex/remove-preview-handoff 分支，预览测试已写好，6 个预期红灯、1 个链接测试显式忽略，删除测试 6 个通过。preview_path 已创建 Vec 并调用 collect_preview，collect_preview 仍为空实现。我学习 Rust，生产代码由我写；你负责解释标准库/API、文件与模块组织、完整规则的测试与伪代码、审查和局部运行测试，不要替我实现。继续指导我完成预览结果和目录展开规则。
