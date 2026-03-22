use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "logseq")]
#[command(about = "Transform Logseq markdown into an AST", long_about = None)]
struct Args {
    /// Input markdown file path, or '-' for STDIN
    input: String,

    /// Output format (currently only json)
    #[arg(long, default_value = "json")]
    format: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let input = if args.input == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&args.input)?
    };

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
