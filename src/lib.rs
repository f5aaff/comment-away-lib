use libloading::{Library, Symbol};
use object::{Object, ObjectSymbol};
use std::ffi::CString;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Parser};

pub mod config;
pub mod util;

use anyhow::{Context, Result};
// loads the library from the given shared object, wrapped to produce a Result.
pub fn load_lib_so(path: String) -> Result<Library, anyhow::Error> {
    let library = unsafe { Library::new(path)? };

    Ok(library)
}

// programmatically find the tree_sitter function, to use for the Language
// load function. This searches the shared object for symbols containing both the
// target langage and tree_sitter, and returns the shortest result, as that is likely
// to be the target (e.g tree_sitter_javascript or python_tree_sitter)
pub fn find_tree_sitter_function(path: &str, target: &str) -> Result<Option<String>> {
    // Read the shared library file
    let file =
        fs::read(path).with_context(|| format!("Failed to read shared library file: {}", path))?;

    // Parse the shared library as an object file
    let obj_file = object::File::parse(&*file)
        .with_context(|| format!("Failed to parse object file: {}", path))?;

    // Collect all symbol names containing both "tree_sitter" and the target substring
    let mut symbols: Vec<String> = obj_file
        .symbols()
        .filter_map(|symbol| symbol.name().ok()) // Filter out invalid symbol names
        .filter(|name| name.contains("tree_sitter") && name.contains(target))
        .map(|name| name.to_string())
        .collect();

    // Return the shortest symbol or None if the list is empty
    symbols.sort_by_key(|name| name.len());
    Ok(symbols.into_iter().next())
}

// load the language from the library, converting the name str to a CString
// for the null terminating byte.
pub fn load_language(library: &Library, name: &str) -> Result<Language, anyhow::Error> {
    // Append a null terminator to the name and convert it to CString
    let c_name = CString::new(name)?;

    // Get the byte slice with a null terminator
    let bytes = c_name.as_bytes_with_nul();

    let language: Language = unsafe {
        let func: Symbol<unsafe extern "C" fn() -> Language> = library
            .get(bytes)
            .expect("Failed to load language function");
        func()
    };

    Ok(language)
}

// creates a parser from the given language, wrapped to produce a Result.
pub fn create_parser(language: Language) -> Result<Parser, anyhow::Error> {
    let mut parser = Parser::new();
    parser.set_language(&language)?;

    Ok(parser)
}

// generates a Tree_sitter::Tree from the source code, using the parser.
pub fn gen_tree(mut parser: Parser, source_code: &str) -> tree_sitter::Tree {
    let tree = parser
        .parse(source_code, None)
        .expect("Failed to parse source code");
    tree
}

pub fn strip_nodes_no_ws(
    node: tree_sitter::Node,
    source_code: &mut String,
    comment_types: &Vec<String>,
    offset: &mut isize, // Tracks the net offset created by the removals
) {
    if comment_types.contains(&node.kind().to_string()) {
        let start = node.start_byte();
        let end = node.end_byte();

        // Adjust the byte range considering the current offset
        let adjusted_start = (start as isize + *offset) as usize;
        let adjusted_end = (end as isize + *offset) as usize;

        let comment_text = &source_code[adjusted_start..adjusted_end];

        // Filter out all characters except newlines
        let replacement: String = comment_text.chars().filter(|&c| c == '\n').collect();

        // Replace the comment with the replacement string
        source_code.replace_range(adjusted_start..adjusted_end, &replacement);

        // Update the offset after removal
        let old_len = adjusted_end - adjusted_start;
        let new_len = replacement.len();
        *offset += new_len as isize - old_len as isize;
    }

    for child in node.children(&mut node.walk()) {
        strip_nodes_no_ws(child, source_code, comment_types, offset);
    }
}

// traverses the tree, finding comments.
pub fn strip_nodes(node: tree_sitter::Node, source_code: &mut String, comment_types: &Vec<String>) {
    if comment_types.contains(&node.kind().to_string()) {
        let start = node.start_byte();
        let end = node.end_byte();
        let comment_length = end - start;
        source_code.replace_range(start..end, &" ".repeat(comment_length));
    }

    for child in node.children(&mut node.walk()) {
        strip_nodes(child, source_code, &comment_types);
    }
}

// Helper function to read a file into a String
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}
