// ---------------- [ File: workspacer-syntax/src/generate_struct_signature.rs ]
crate::ix!();

#[derive(Debug, Clone)]
pub struct StructSignatureGenerator(ast::Struct);

impl GenerateSignature for ast::Struct {
    fn generate_signature_with_opts(&self, opts: &SignatureOptions) -> String {
        trace!("Generating signature for ast::Struct with opts: {:?}", opts);

        let doc_text = if *opts.include_docs() {
            extract_docs(&self.syntax())
                .map(|d| format!("{}\n", d))
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name_str = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_struct>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        // We add a space before the "where_clause" if it's non-empty:
        let raw_where = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if raw_where.is_empty() {
            "".to_string()
        } else {
            format!(" {}", raw_where)
        };

        // If fully_expand is true, gather and align the actual fields; otherwise, we display a placeholder.
        let fields_text = if *opts.fully_expand() {
            if let Some(fl) = self.field_list() {
                match fl {
                    ast::FieldList::RecordFieldList(rfl) => {
                        align_record_fields(&rfl)
                    }
                    ast::FieldList::TupleFieldList(tfl) => {
                        align_tuple_fields(&tfl)
                    }
                }
            } else {
                // no fields => e.g. `struct Foo;`
                ";".to_string()
            }
        } else {
            // minimal or placeholder approach
            "{ /* fields omitted */ }".to_string()
        };

        let core = format!("{vis_str}struct {name_str}{generic_params_raw}{where_clause} {fields_text}");

        let final_sig = format!("{doc_text}{core}");
        post_process_spacing(&final_sig)
    }
}

/// Aligns record fields so that field names and colons line up in a neat column:
/// ```
/// {
///     path:                    Option<PathBuf>,
///     include_private:         bool,
///     show_items_with_no_data: bool,
/// }
/// ```
fn align_record_fields(rfl: &ast::RecordFieldList) -> String {
    // 1) Gather the fields into a vector of (field_name, field_type)
    let mut fields_info = Vec::new();
    for field in rfl.fields() {
        let fname = field
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        let fty = field
            .ty()
            .map(|t| t.syntax().text().to_string())
            .unwrap_or_default();

        fields_info.push((fname, fty));
    }

    if fields_info.is_empty() {
        return "{ }".to_string();
    }

    // 2) Find the max length of any field name. We'll align the colon after that.
    let max_name_len = fields_info.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    // 3) Build the final lines, each with `    {field_name}{spaces}: {field_type},`
    let mut lines = Vec::new();
    for (name, ftype) in fields_info {
        let space_count = if max_name_len > name.len() {
            max_name_len - name.len()
        } else {
            0
        };
        let spacing = " ".repeat(space_count);
        // so we get something like: `    path{...}: Option<PathBuf>,`
        let line = format!("    {name}:{spacing} {ftype},");
        lines.push(line);
    }

    let joined = lines.join("\n");
    format!("{{\n{}\n}}", joined)
}

/// Aligns tuple fields so that the entire `vis + type` is lined up in a neat column. For example:
/// ```
/// (
///     pub(in xyz) i32,
///     bool,
///     AnotherType,
/// );
/// ```
fn align_tuple_fields(tfl: &ast::TupleFieldList) -> String {
    // 1) Gather the fields into a vector of strings, each "vis + type"
    let mut fields_info = Vec::new();
    for field in tfl.fields() {
        let vis = field
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();
        let fty = field
            .ty()
            .map(|t| t.syntax().text().to_string())
            .unwrap_or_default();

        // Combine them into one string: e.g. "pub(in foo) MyType"
        let combined = format!("{}{}", vis, fty);
        fields_info.push(combined);
    }

    if fields_info.is_empty() {
        return "();".to_string();
    }

    // 2) Find the max length of that combined "vis + type" text
    let max_field_len = fields_info.iter().map(|s| s.len()).max().unwrap_or(0);

    // 3) Build final lines with left alignment
    let mut lines = Vec::new();
    for combined in fields_info {
        let space_count = if max_field_len > combined.len() {
            max_field_len - combined.len()
        } else {
            0
        };
        let spacing = " ".repeat(space_count);
        // e.g. `    pub(in foo) i32,`
        let line = format!("    {combined}{spacing},");
        lines.push(line);
    }

    let joined = lines.join("\n");
    format!("(\n{}\n);", joined)
}

/// A small helper to do final spacing adjustments if desired.
fn post_process_spacing(sig: &str) -> String {
    // For example, you can remove trailing blank lines or compress double newlines.
    // For this snippet, we just return `sig` as-is.
    sig.to_string()
}
