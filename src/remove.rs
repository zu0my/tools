use std::fs::{remove_dir, remove_dir_all, remove_file, symlink_metadata};
use std::path::{Path, PathBuf};

fn remove_path(path: &Path, recursive: bool) -> std::io::Result<()> {
    let metadata = symlink_metadata(path)?;

    if metadata.is_dir() {
        if recursive {
            remove_dir_all(path)
        } else {
            remove_dir(path)
        }
    } else {
        remove_file(path)
    }
}

fn collect_preview(
    current_path: &Path,
    recursive: bool,
    preview_paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    Ok(())
}

fn preview_path(path: &Path, recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    let mut preview_paths: Vec<PathBuf> = vec![];

    collect_preview(path, recursive, &mut preview_paths)?;

    Ok(preview_paths)
}

#[cfg(test)]
mod tests {
    use std::fs::create_dir;
    use std::fs::{read_to_string, write};
    use std::io::ErrorKind;
    use tempfile::tempdir;

    use super::{preview_path, remove_path};

    #[test]
    fn remove_existing_file_deletes_it() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let file_path = temp_dir.path().join("a.txt");
        write(&file_path, "hello").expect("创建测试文件失败");

        remove_path(&file_path, false).expect("删除测试文件失败");

        assert!(
            !file_path.try_exists().expect("检查删除结果失败"),
            "删除成功后文件仍然存在"
        );
    }

    #[test]
    fn remove_missing_file_returns_not_found() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let file_path = temp_dir.path().join("missing.txt");

        let err = remove_path(&file_path, false).expect_err("删除不存在的文件应返回错误");

        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn remove_empty_directory_deletes_it() {
        let temp_dir = tempdir().expect("创建临时目录失败");

        let empty_dir_path = temp_dir.path().join("empty");

        create_dir(&empty_dir_path).expect("创建 emtpy 目录失败");

        remove_path(&empty_dir_path, false).expect("删除 empty 目录失败");

        assert!(
            !empty_dir_path.try_exists().expect("检查删除结果失败"),
            "删除 empty 目录后，目录仍然存在"
        );
    }

    #[test]
    fn remove_nonempty_directory_returns_error_and_preserves_contents() {
        let temp_dir = tempdir().expect("创建临时目录失败");

        let nonempty_dir_path = temp_dir.path().join("nonempty");

        create_dir(&nonempty_dir_path).expect("创建 nonemtpy 目录失败");

        let file_path = nonempty_dir_path.join("a.txt");

        let file_content = "hello world";

        write(&file_path, file_content).expect("创建测试文件失败");

        let err = remove_path(&nonempty_dir_path, false).expect_err("删除有文件的文件夹应返回错误");

        assert_eq!(err.kind(), ErrorKind::DirectoryNotEmpty);
        assert_eq!(
            read_to_string(&file_path).expect("读取文件内容失败"),
            file_content
        );
    }

    #[test]
    fn remove_nonempty_directory_recursively_deletes_tree() {
        let temp_dir = tempdir().expect("创建临时目录失败");

        let nonempty_dir_path = temp_dir.path().join("nonempty");
        let nested_dir_path = nonempty_dir_path.join("nested");

        create_dir(&nonempty_dir_path).expect("创建 nonemtpy 目录失败");
        create_dir(&nested_dir_path).expect("Failed to create nested");

        let file_a_path = nonempty_dir_path.join("a.txt");
        let file_b_path = nested_dir_path.join("b.txt");

        write(&file_a_path, "hello world").expect("创建测试文件失败");
        write(&file_b_path, "hello world").expect("Failed to create b.txt");

        remove_path(&nonempty_dir_path, true).expect("删除 nonempty 文件失败");
        assert!(!nonempty_dir_path.try_exists().expect("检查结果失败"));
    }

    #[test]
    fn remove_file_with_recursive_enabled_deletes_it() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let file_path = temp_dir.path().join("a.txt");

        write(&file_path, "hello world").expect("Failed to create a.txt");

        remove_path(&file_path, true).expect("Failed to remove a.txt");

        assert!(!file_path.try_exists().expect("Failed to check file"));
    }

    #[test]
    fn preview_file_returns_path_without_deleting_it() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let file_path = temp_dir.path().join("a.txt");
        write(&file_path, "hello").expect("创建测试文件失败");

        for recursive in [false, true] {
            let paths = preview_path(&file_path, recursive).expect("预览文件失败");

            assert_eq!(paths, vec![file_path.clone()]);
            assert!(
                file_path.try_exists().expect("检查文件是否存在失败"),
                "预览不应删除文件"
            );
        }
    }

    #[test]
    fn preview_empty_directory_returns_directory_without_deleting_it() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let directory_path = temp_dir.path().join("empty");
        create_dir(&directory_path).expect("创建空目录失败");

        for recursive in [false, true] {
            let paths = preview_path(&directory_path, recursive).expect("预览空目录失败");

            assert_eq!(paths, vec![directory_path.clone()]);
            assert!(
                directory_path.try_exists().expect("检查目录是否存在失败"),
                "预览不应删除空目录"
            );
        }
    }

    #[test]
    fn preview_nonempty_directory_without_recursion_returns_error() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let directory_path = temp_dir.path().join("nonempty");
        create_dir(&directory_path).expect("创建目标目录失败");
        let file_path = directory_path.join("a.txt");
        write(&file_path, "hello").expect("创建测试文件失败");

        let error =
            preview_path(&directory_path, false).expect_err("未启用递归时，预览非空目录应返回错误");

        assert_eq!(error.kind(), ErrorKind::DirectoryNotEmpty);
        for path in [&directory_path, &file_path] {
            assert!(
                path.try_exists().expect("检查目标是否存在失败"),
                "预览失败也不应删除目标：{}",
                path.display()
            );
        }
    }

    #[test]
    fn preview_recursive_directory_returns_all_paths_in_order() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let directory_path = temp_dir.path().join("target");
        let nested_path = directory_path.join("nested");
        create_dir(&directory_path).expect("创建目标目录失败");
        create_dir(&nested_path).expect("创建嵌套目录失败");
        let file_a_path = directory_path.join("a.txt");
        let file_b_path = nested_path.join("b.txt");
        write(&file_a_path, "a").expect("创建 a.txt 失败");
        write(&file_b_path, "b").expect("创建 b.txt 失败");

        let paths = preview_path(&directory_path, true).expect("递归预览目录失败");

        assert_eq!(
            paths,
            vec![
                directory_path.clone(),
                file_a_path.clone(),
                nested_path.clone(),
                file_b_path.clone(),
            ]
        );
        for path in [&directory_path, &file_a_path, &nested_path, &file_b_path] {
            assert!(
                path.try_exists().expect("检查目标是否存在失败"),
                "预览不应删除目标：{}",
                path.display()
            );
        }
    }

    #[test]
    fn preview_directory_sorts_siblings() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let directory_path = temp_dir.path().join("target");
        create_dir(&directory_path).expect("创建目标目录失败");
        let file_z_path = directory_path.join("z.txt");
        let file_a_path = directory_path.join("a.txt");
        write(&file_z_path, "z").expect("创建 z.txt 失败");
        write(&file_a_path, "a").expect("创建 a.txt 失败");

        let paths = preview_path(&directory_path, true).expect("递归预览目录失败");

        assert_eq!(
            paths,
            vec![
                directory_path.clone(),
                file_a_path.clone(),
                file_z_path.clone()
            ]
        );
        for path in [&directory_path, &file_a_path, &file_z_path] {
            assert!(
                path.try_exists().expect("检查目标是否存在失败"),
                "预览不应删除目标：{}",
                path.display()
            );
        }
    }

    #[test]
    fn preview_missing_path_returns_not_found() {
        let temp_dir = tempdir().expect("创建临时目录失败");
        let missing_path = temp_dir.path().join("missing.txt");

        for recursive in [false, true] {
            let error =
                preview_path(&missing_path, recursive).expect_err("预览不存在的路径应返回错误");

            assert_eq!(error.kind(), ErrorKind::NotFound);
            assert!(
                !missing_path.try_exists().expect("检查目标是否存在失败"),
                "预览不应创建目标"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "需要 Windows 开发者模式或创建符号链接权限；使用 --ignored 单独运行"]
    fn preview_symlink_does_not_expand_target() {
        use std::fs::symlink_metadata;
        use std::os::windows::fs::symlink_dir;

        let temp_dir = tempdir().expect("创建临时目录失败");
        let target_path = temp_dir.path().join("target");
        let link_path = temp_dir.path().join("link");
        create_dir(&target_path).expect("创建链接目标目录失败");
        let file_path = target_path.join("a.txt");
        write(&file_path, "hello").expect("创建链接目标文件失败");
        symlink_dir(&target_path, &link_path)
            .expect("创建目录符号链接失败；此测试需要 Windows 开发者模式或创建链接权限");

        for recursive in [false, true] {
            let paths = preview_path(&link_path, recursive).expect("预览目录符号链接失败");

            assert_eq!(paths, vec![link_path.clone()]);
            assert!(
                symlink_metadata(&link_path)
                    .expect("读取链接元数据失败")
                    .file_type()
                    .is_symlink(),
                "预览不应删除或替换链接"
            );
            assert!(target_path.try_exists().expect("检查链接目标目录失败"));
            assert!(file_path.try_exists().expect("检查链接目标文件失败"));
        }
    }
}
