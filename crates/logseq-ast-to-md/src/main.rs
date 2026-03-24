use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "logseq-ast-to-md")]
#[command(about = "Convert logseq-ast JSON output into blog-style Markdown", long_about = None)]
struct Args {
    /// Input JSON file path.
    ///
    /// If omitted, reads from STDIN.
    /// Use "-" explicitly to force STDIN.
    input: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input_arg = args.input.as_deref().unwrap_or("-");

    let json = if input_arg == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(input_arg)?
    };

    let doc: logseq_core::ast::Document = serde_json::from_str(&json)?;

    let md = logseq_core::blog::render_blog_markdown(&doc);
    print!("{md}");

    Ok(())
}
