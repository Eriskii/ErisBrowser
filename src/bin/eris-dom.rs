//! Tree-only conformance adapter. Input is never executed or fetched.
use eris::dom::{Document, MAX_NODES, NodeKind};
use std::io::{self, Read, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("eris-dom: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut scripting = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scripting" => {
                scripting = match args.next().as_deref() {
                    Some("enabled") => true,
                    Some("disabled") => false,
                    _ => return Err("--scripting requires enabled or disabled".into()),
                }
            }
            "--help" => {
                println!(
                    "Usage: eris-dom [--scripting enabled|disabled] < input.html\nOutputs the actual parsed tree in WPT tree-construction format.\nSets the parser scripting flag; scripts are never executed.\nFragment parsing and parse-error reporting are not implemented by this adapter."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    const MAX_SOURCE: usize = 8 * 1024 * 1024;
    let mut source = String::new();
    io::stdin()
        .take(MAX_SOURCE as u64 + 1)
        .read_to_string(&mut source)
        .map_err(|e| e.to_string())?;
    if source.len() > MAX_SOURCE {
        return Err("HTML exceeds adapter source limit".into());
    }
    let document = Document::parse_with_scripting(&source, scripting);
    io::stdout()
        .write_all(serialize(&document)?.as_bytes())
        .map_err(|e| e.to_string())
}

fn serialize(document: &Document) -> Result<String, String> {
    use std::fmt::Write;
    let mut out = String::new();
    let mut pending = document.nodes[document.root]
        .children
        .iter()
        .rev()
        .map(|node| (*node, 0usize))
        .collect::<Vec<_>>();
    let mut visited = 0;
    while let Some((id, depth)) = pending.pop() {
        visited += 1;
        if visited > MAX_NODES || depth > eris::dom::MAX_DEPTH {
            return Err("tree exceeds adapter traversal limit".into());
        }
        let node = document
            .nodes
            .get(id)
            .ok_or("invalid DOM child reference")?;
        let prefix = format!("| {}", "  ".repeat(depth));
        match &node.kind {
            NodeKind::Document => return Err("nested document node".into()),
            NodeKind::Element(element) => {
                writeln!(out, "{prefix}<{}>", element.tag).map_err(|e| e.to_string())?;
                let mut attributes = element.attrs.iter().collect::<Vec<_>>();
                attributes
                    .sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
                for (name, value) in attributes {
                    writeln!(out, "{prefix}  {name}=\"{value}\"").map_err(|e| e.to_string())?;
                }
            }
            NodeKind::Text(text) => {
                writeln!(out, "{prefix}\"{text}\"").map_err(|e| e.to_string())?;
            }
            NodeKind::Comment(data) => {
                writeln!(out, "{prefix}<!-- {data} -->").map_err(|e| e.to_string())?;
            }
            NodeKind::ProcessingInstruction { target, data } => {
                writeln!(out, "{prefix}<?{target} {data}?>").map_err(|e| e.to_string())?;
            }
            NodeKind::Doctype(doctype) => {
                write!(out, "{prefix}<!DOCTYPE {}", doctype.name).map_err(|e| e.to_string())?;
                let public = doctype.public_id.as_deref().unwrap_or("");
                let system = doctype.system_id.as_deref().unwrap_or("");
                if !public.is_empty() || !system.is_empty() {
                    write!(out, " \"{public}\" \"{system}\"").map_err(|e| e.to_string())?;
                }
                out.push_str(">\n");
            }
        }
        if out.len() > 32 * 1024 * 1024 {
            return Err("tree serialization exceeds output limit".into());
        }
        pending.extend(node.children.iter().rev().map(|child| (*child, depth + 1)));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serializer_does_not_escape_or_discard_node_data() {
        let mut document = Document::parse("");
        let body = document.query_selector("body").unwrap();
        let text = document.create_text_node("a\n\"b");
        document.append_child(body, text);
        let comment = document.create_comment("comment\nline");
        document.append_child(body, comment);
        let dump = serialize(&document).unwrap();
        assert!(dump.contains("|     \"a\n\"b\"\n"), "{dump}");
        assert!(dump.contains("|     <!-- comment\nline -->\n"), "{dump}");
    }
}
