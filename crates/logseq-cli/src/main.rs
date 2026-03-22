use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "logseq-ast")]
#[command(about = "Transform Logseq markdown into an AST", long_about = None)]
struct Args {
    /// Input markdown file path.
    ///
    /// If omitted, reads from STDIN (Unix filter style).
    /// Use "-" explicitly to force STDIN.
    input: Option<String>,

    /// Output format (currently only json)
    #[arg(long, default_value = "json")]
    format: String,

    /// Print debug tokenization output to STDERR (for troubleshooting).
    #[arg(long)]
    debug_tokens: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let input_arg = args.input.as_deref().unwrap_or("-");

    let input = if input_arg == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(input_arg)?
    };

    if args.debug_tokens {
        eprintln!("--- debug_tokens (tokenize_inline per non-empty line) ---");
        for (i, line) in input.lines().enumerate() {
            let line_no = i + 1;
            if line.trim().is_empty() {
                continue;
            }
            let toks = logseq_core::tokenize::tokenize_inline(line);
            // Keep noise down: print only when something was recognized.
            if toks.len() > 1 {
                eprintln!("line {line_no}: {toks:?}");
            }
        }
        eprintln!("--- end debug_tokens ---");
    }

    let doc = logseq_core::parse::parse(&input).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    match args.format.as_str() {
        "json" => {
            let s = serde_json::to_string_pretty(&doc)?;
            println!("{s}");
        }
        other => anyhow::bail!("unsupported format: {other}"),
    }

    Ok(())
}
