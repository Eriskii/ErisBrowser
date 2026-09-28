//! Tree-only conformance adapter. Input is never executed or fetched.
use eris::dom::{AttributeNamespace, Document, MAX_NODES, Namespace, NodeKind};
use std::io::{self, Read, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("eris-dom: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut scripting = false;
    let mut fragment = None;
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
            "--fragment" => {
                fragment = Some(args.next().ok_or("--fragment requires a context name")?);
            }
            "--help" => {
                println!(
                    "Usage: eris-dom [--scripting enabled|disabled] [--fragment CONTEXT] < input.html\nOutputs the actual parsed tree in WPT tree-construction format.\nCONTEXT is an HTML local name, 'svg NAME', or 'math NAME'.\nSets the parser scripting flag; scripts are never executed.\nParse-error reporting is not implemented."
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
    let document = if let Some(context) = fragment {
        fragment_document(&source, &context, scripting)?
    } else {
        Document::parse_with_scripting(&source, scripting)
    };
    io::stdout()
        .write_all(serialize(&document)?.as_bytes())
        .map_err(|e| e.to_string())
}

fn fragment_document(source: &str, context: &str, scripting: bool) -> Result<Document, String> {
    let (namespace, tag) = if let Some(tag) = context.strip_prefix("svg ") {
        (Namespace::Svg, tag)
    } else if let Some(tag) = context.strip_prefix("math ") {
        (Namespace::MathMl, tag)
    } else {
        (Namespace::Html, context)
    };
    if tag.is_empty() || tag.len() > 256 || tag.chars().any(char::is_whitespace) {
        return Err("invalid fragment context name".into());
    }
    // The fixture context is created in a standards-mode owner document.
    let mut document = Document::parse_with_scripting("<!doctype html>", scripting);
    let element = document.create_element_ns(namespace, tag);
    if document.tag(element) != Some(tag) {
        return Err("invalid fragment context name".into());
    }
    document.parse_fragment(element, source)
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
            NodeKind::DocumentFragment { .. } => {
                writeln!(out, "{prefix}content").map_err(|e| e.to_string())?;
            }
            NodeKind::Element(element) => {
                let namespace = match element.namespace {
                    Namespace::Html => "",
                    Namespace::Svg => "svg ",
                    Namespace::MathMl => "math ",
                };
                writeln!(out, "{prefix}<{namespace}{}>", element.tag).map_err(|e| e.to_string())?;
                let mut attributes = element
                    .attrs
                    .iter()
                    .map(|(name, value)| {
                        let (namespace, local_name) = match element.attr_namespaces.get(name) {
                            Some(AttributeNamespace::XLink) => {
                                ("xlink ", name.strip_prefix("xlink:").unwrap_or(name))
                            }
                            Some(AttributeNamespace::Xml) => {
                                ("xml ", name.strip_prefix("xml:").unwrap_or(name))
                            }
                            Some(AttributeNamespace::Xmlns) => {
                                ("xmlns ", name.strip_prefix("xmlns:").unwrap_or(name))
                            }
                            None => ("", name.as_str()),
                        };
                        (format!("{namespace}{local_name}"), value)
                    })
                    .collect::<Vec<_>>();
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
        if let NodeKind::Element(element) = &node.kind
            && let Some(content) = element.template_contents
        {
            pending.push((content, depth + 1));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adapter_emits_fragment_children_and_preserves_context_namespace() {
        for (context, source, expected) in [
            (
                "table",
                "<td>x",
                "| <tbody>\n|   <tr>\n|     <td>\n|       \"x\"\n",
            ),
            ("textarea", "<b>&amp;</textarea>", "| \"<b>&</textarea>\"\n"),
            (
                "svg svg",
                "<lineargradient xlink:href='#x'/>",
                "| <svg linearGradient>\n|   xlink href=\"#x\"\n",
            ),
            (
                "math mi",
                "<mglyph/><b>x</b>",
                "| <math mglyph>\n| <b>\n|   \"x\"\n",
            ),
        ] {
            let document = fragment_document(source, context, false).unwrap();
            assert_eq!(serialize(&document).unwrap(), expected, "{context}");
        }
        for invalid in ["", "svg ", "xml element", "div\nbody"] {
            assert!(fragment_document("x", invalid, false).is_err(), "{invalid}");
        }
    }
    #[test]
    fn template_serialization_visits_real_content_fragments() {
        let document = Document::parse("<template><tr><td>x</template>");
        let template = document.query_selector("template").unwrap();
        let fragment = document.template_contents(template).unwrap();
        assert!(document.nodes[template].children.is_empty());
        assert!(document.nodes[fragment].parent.is_none());
        assert_eq!(
            serialize(&document).unwrap(),
            "| <html>\n|   <head>\n|     <template>\n|       content\n|         <tr>\n|           <td>\n|             \"x\"\n|   <body>\n"
        );
        let fragment = fragment_document("<tr><td>x", "template", false).unwrap();
        assert_eq!(
            serialize(&fragment).unwrap(),
            "| <tr>\n|   <td>\n|     \"x\"\n"
        );
    }
    #[test]
    fn foreign_namespaces_and_adjusted_attribute_names_are_sorted_exactly() {
        let document = Document::parse(
            "<svg xml:base xml:lang xml:space xml:baaah definitionurl xmlns='u' xmlns:xlink='x'><foreignObject><p>html</p></foreignObject></svg>",
        );
        let dump = serialize(&document).unwrap();
        assert_eq!(
            dump,
            "| <html>\n|   <head>\n|   <body>\n|     <svg svg>\n|       definitionurl=\"\"\n|       xml lang=\"\"\n|       xml space=\"\"\n|       xml:baaah=\"\"\n|       xml:base=\"\"\n|       xmlns xlink=\"x\"\n|       xmlns xmlns=\"u\"\n|       <svg foreignObject>\n|         <p>\n|           \"html\"\n"
        );
    }
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
