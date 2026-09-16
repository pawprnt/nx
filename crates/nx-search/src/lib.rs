use clap::Args;
use color_eyre::eyre::Result;
use nx_core::log;
use yansi::Paint;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: Option<String>,

    /// Use the search.nixos.org web API instead of nix search
    #[arg(long)]
    pub web: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Show descriptions
    #[arg(long)]
    pub description: bool,
}

pub fn run(args: SearchArgs) -> Result<()> {
    let query = args.query.unwrap_or_default();

    if query.is_empty() {
        log::warn("Please provide a search query: nx search <query>");
        return Ok(());
    }

    if args.web {
        search_web(&query, args.json, args.description)
    } else {
        search_local(&query, args.json, args.description)
    }
}

fn search_local(query: &str, json: bool, description: bool) -> Result<()> {
    log::info(&format!("Searching nixpkgs for '{}'...", query));

    let mut cmd = std::process::Command::new("nix");
    cmd.args(["search", "nixpkgs", query, "--json"]);

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error(&format!("Search failed: {}", stderr.trim()));
        return Ok(());
    }

    if json {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

    let results: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let default_map = serde_json::Map::new();
    let results = results.as_object().unwrap_or(&default_map);

    if results.is_empty() {
        log::warn("No results found");
        return Ok(());
    }

    eprintln!();
    for (name, info) in results {
        let version = info["version"].as_str().unwrap_or("?");
        let desc = info["description"].as_str().unwrap_or("");

        if description {
            eprintln!("  {} {}", name.green().bold(), version.dim());
            eprintln!("    {}", desc);
        } else {
            eprintln!("  {} {}", name.green().bold(), version.dim());
        }
    }
    eprintln!();
    eprintln!("  {} results found", results.len().to_string().green());
    eprintln!();

    Ok(())
}

#[tokio::main]
async fn search_web(query: &str, json: bool, description: bool) -> Result<()> {
    log::info(&format!("Searching search.nixos.org for '{}'...", query));

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "query": {
            "bool": {
                "must": [
                    {
                        "multi_match": {
                            "query": query,
                            "fields": [
                                "package_pname^10",
                                "package_pname.autocomplete^5",
                                "package_name^2",
                                "package_name.autocomplete",
                                "package_description",
                                "package_description.autocomplete"
                            ],
                            "type": "bool_prefix",
                            "operator": "and"
                        }
                    }
                ]
            }
        },
        "size": 20,
        "_source": ["package_pname", "package_version", "package_description"]
    });

    let resp = client
        .post("https://search.nixos.org/backend/latest-32/_search")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let data: serde_json::Value = resp.json().await?;
    let empty_vec = vec![];
    let hits = data["hits"]["hits"].as_array().unwrap_or(&empty_vec);

    if hits.is_empty() {
        log::warn("No results found");
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(hits)?);
        return Ok(());
    }

    eprintln!();
    for hit in hits {
        let src = &hit["_source"];
        let name = src["package_pname"].as_str().unwrap_or("?");
        let version = src["package_version"].as_str().unwrap_or("?");
        let desc = src["package_description"].as_str().unwrap_or("");

        if description {
            eprintln!("  {} {}", name.green().bold(), version.dim());
            eprintln!("    {}", desc);
        } else {
            eprintln!("  {} {}", name.green().bold(), version.dim());
        }
    }
    eprintln!();
    eprintln!("  {} results found", hits.len().to_string().green());
    eprintln!();

    Ok(())
}
