crate::ix!();

/// Groups items by file path, then prints them similarly to the old approach.
pub fn build_filtered_grouped_by_file_string(options: &ShowFlags, cci: &ConsolidatedCrateInterface) -> String {
    trace!("Grouping items by file in build_filtered_grouped_by_file_string");

    let mut items_by_file: HashMap<PathBuf, Vec<ConsolidatedItem>> = HashMap::new();

    fn push_item(map: &mut HashMap<PathBuf, Vec<ConsolidatedItem>>, path: &PathBuf, item: ConsolidatedItem) {
        map.entry(path.clone()).or_insert_with(Vec::new).push(item);
    }

    for fn_item in cci.fns() {
        let p = fn_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Fn(fn_item.clone()));
    }
    for st_item in cci.structs() {
        let p = st_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Struct(st_item.clone()));
    }
    for en_item in cci.enums() {
        let p = en_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Enum(en_item.clone()));
    }
    for tr_item in cci.traits() {
        let p = tr_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Trait(tr_item.clone()));
    }
    for ta_item in cci.type_aliases() {
        let p = ta_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::TypeAlias(ta_item.clone()));
    }
    for mac_item in cci.macros() {
        let p = mac_item.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Macro(mac_item.clone()));
    }
    for ib in cci.impls() {
        let p = ib.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::ImplBlock(ib.clone()));
    }
    for mo in cci.modules() {
        let p = mo.file_path().clone();
        push_item(&mut items_by_file, &p, ConsolidatedItem::Module(mo.clone()));
    }

    if items_by_file.is_empty() {
        if options.show_items_with_no_data() {
            return "<no-data-for-crate>\n".to_string();
        } else {
            return String::new();
        }
    }

    let mut file_paths: Vec<_> = items_by_file.keys().cloned().collect();
    file_paths.sort();

    let mut out = String::new();
    for (i, fpath) in file_paths.iter().enumerate() {
        let items = items_by_file.get(fpath).unwrap();
        if items.is_empty() {
            if options.show_items_with_no_data() {
                out.push_str(&format!("--- [File: {}] ---\n", fpath.display()));
                out.push_str("<no-data-for-file>\n\n");
            }
            continue;
        }
        out.push_str(&format!("--- [File: {}] ---\n", fpath.display()));
        for (j, it) in items.iter().enumerate() {
            out.push_str(&format!("{}", it));
            out.push('\n');
            if j + 1 < items.len() {
                out.push('\n');
            }
        }
        if i + 1 < file_paths.len() {
            out.push('\n');
        }
    }
    out
}
