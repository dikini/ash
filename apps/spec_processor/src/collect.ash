use types::FileTree;
use io::dir::read_dir;
use io::meta::metadata;
use io::path::PathBuf;

fn collect_files(acc: List<String>, path: String) -> List<String> {
    let entries = read_dir(PathBuf { inner: path });
    process_entries(acc, entries)
}

fn process_entries(acc: List<String>, entries: List<String>) -> List<String> {
    if len(entries) == 0 then
        acc
    else {
        let entry = head(entries);
        let rest = tail(entries);
        let meta = metadata(PathBuf { inner: entry });
        match meta {
            Metadata { is_dir: true, .. } => {
                let with_sub = collect_files(acc, entry);
                process_entries(with_sub, rest)
            },
            _ => {
                let new_acc = append(acc, entry);
                process_entries(new_acc, rest)
            }
        }
    }
}

pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    FileTree { spec_files: filter(all, |p| => starts_with(p, "SPEC-") && ends_with(p, ".md")), plan_files: filter(all, |p| => starts_with(p, "PLAN-") && ends_with(p, ".md")), example_files: filter(all, |p| => ends_with(p, ".ash")), changelog_files: filter(all, |p| => ends_with(p, "CHANGELOG.md")) }
}
