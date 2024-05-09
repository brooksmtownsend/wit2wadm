use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use indexmap::IndexSet;
use wit_parser::{PackageName, SourceMap, UnresolvedPackage};

use crate::read_file;
use crate::wasi::filesystem::types::{
    Descriptor, DescriptorFlags, DescriptorType, ErrorCode, OpenFlags, PathFlags,
};

fn parse_dir(dir: &Descriptor, dir_path: &Path) -> Result<UnresolvedPackage, String> {
    let entries = dir
        .read_directory()
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    let mut map = SourceMap::default();
    while let Some(res) = entries.read_directory_entry().transpose() {
        let entry = res.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        if matches!(
            entry.type_,
            DescriptorType::RegularFile | DescriptorType::SymbolicLink
        ) {
            let path = dir_path.join(&entry.name);
            let ext = path.extension().unwrap_or_default();
            if ext != "wit" {
                continue;
            }
            let file = dir
                .open_at(
                    PathFlags::SYMLINK_FOLLOW,
                    &entry.name,
                    OpenFlags::empty(),
                    DescriptorFlags::READ,
                )
                .map_err(|e| format!("Failed to open file {}: {}", path.display(), e))?;
            let contents = read_file(file)
                .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
            map.push(
                &path,
                String::from_utf8(contents)
                    .map_err(|e| format!("File is not valid UTF-8 text: {}", e))?,
            );
        }
    }
    map.parse().map_err(|e| {
        format!(
            "Unable to parse WIT package {}: {:?}",
            dir_path.display(),
            e
        )
    })
}

// This is copied straight from wit-parser 0.207 and adapted to match our needs for loading a wit
// directory within a component. Ideally we should find a way to upstream this back

pub fn push_dir(dir: Descriptor, path: &Path) -> Result<Vec<UnresolvedPackage>, String> {
    let pkg = parse_dir(&dir, path)
        .map_err(|e| format!("failed to parse package {}: {:?}", path.display(), e))?;

    let deps_dir = path.join("deps");
    let mut deps = match dir.open_at(
        PathFlags::empty(),
        "deps",
        OpenFlags::DIRECTORY,
        DescriptorFlags::MUTATE_DIRECTORY | DescriptorFlags::READ,
    ) {
        Ok(next_dir) => parse_deps_dir(next_dir, &deps_dir)?,
        Err(ErrorCode::NoEntry) => BTreeMap::default(),
        Err(e) => return Err(format!("Failed to open deps directory: {}", e)),
    };

    // Perform a simple topological sort which will bail out on cycles
    // and otherwise determine the order that packages must be added to
    // this `Resolve`.
    let mut order = IndexSet::new();
    let mut visiting = HashSet::new();
    for pkg in deps.values().chain([&pkg]) {
        visit(pkg, &deps, &mut order, &mut visiting)?;
    }
    order.insert(pkg.name.clone());
    deps.insert(pkg.name.clone(), pkg);

    let mut output = Vec::new();
    for name in order {
        output.push(deps.remove(&name).unwrap())
    }

    return Ok(output);

    fn visit<'a>(
        pkg: &'a UnresolvedPackage,
        deps: &'a BTreeMap<PackageName, UnresolvedPackage>,
        order: &mut IndexSet<PackageName>,
        visiting: &mut HashSet<&'a PackageName>,
    ) -> Result<(), String> {
        if order.contains(&pkg.name) {
            return Ok(());
        }
        for (dep, _) in pkg.foreign_deps.iter() {
            if !visiting.insert(dep) {
                return Err(format!("package {} depends on itself", pkg.name));
            }
            if let Some(dep) = deps.get(dep) {
                visit(dep, deps, order, visiting)?;
            }
            assert!(visiting.remove(dep));
        }
        assert!(order.insert(pkg.name.clone()));
        Ok(())
    }
}

fn parse_deps_dir(
    dir: Descriptor,
    path: &Path,
) -> Result<BTreeMap<PackageName, UnresolvedPackage>, String> {
    let mut ret = BTreeMap::new();

    let entry_stream = dir
        .read_directory()
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    let mut entries = Vec::new();
    while let Some(res) = entry_stream.read_directory_entry().transpose() {
        let entry = res.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        entries.push(entry);
    }

    entries.sort_by_key(|en| en.name.clone());
    for dep in entries {
        let p = path.join(&dep.name);

        // TODO: This doesn't handle symlinks like the original code
        let pkg = if dep.type_ == DescriptorType::Directory {
            // If this entry is a directory or a symlink point to a
            // directory then always parse it as an `UnresolvedPackage`
            // since it's intentional to not support recursive `deps`
            // directories.
            let next_dir = dir
                .open_at(
                    PathFlags::empty(),
                    &dep.name,
                    OpenFlags::DIRECTORY,
                    DescriptorFlags::READ,
                )
                .map_err(|e| format!("failed to open deps directory {}: {}", path.display(), e))?;
            parse_dir(&next_dir, &p)
                .map_err(|e| format!("failed to parse package {}: {e:?}", path.display()))?
        } else {
            // We're eliding what wit-parser does with trying to load the individual file
            continue;
        };
        let prev = ret.insert(pkg.name.clone(), pkg);
        if let Some(prev) = prev {
            return Err(format!(
                "duplicate definitions of package `{}` found",
                prev.name
            ));
        }
    }
    Ok(ret)
}
