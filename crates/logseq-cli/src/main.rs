use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "logseq")]
#[command(about = "Transform Logseq markdown into an AST", long_about = None)]
struct Args {
    /// Input markdown file path
    input: std::path::PathBuf,

    /// Output format (currently only json)
    #[arg(long, default_value = "json")]
    format: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(&args.input)?;

    let doc = logseq_core::parse::parse(&input)?;

    match args.format.as_str() {
        "json" => {
            let s = serde_json::to_string_pretty(&doc)?;
            println!("{s}");
        }
        other => anyhow::bail!("unsupported format: {other}"),
    }

    Ok(())
}
