use super::color::{CYAN_ANSI, PRIMARY_ANSI};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const TAGLINE: &str = "Host, container, and SSH operations for MCP agents";

struct CommandDoc {
    name: &'static str,
    summary: &'static str,
    usage: &'static [&'static str],
}

struct NestedCommandDoc {
    path: &'static str,
    summary: &'static str,
    usage: &'static [&'static str],
}

struct UsageSection {
    title: &'static str,
    lines: &'static [&'static str],
}

const SECTIONS: &[(&str, &[&str])] = &[
    ("Flux", &["flux"]),
    ("Scout", &["scout"]),
    (
        "Runtime & Setup",
        &["serve", "mcp", "doctor", "watch", "setup", "help"],
    ),
];

const CATALOG: &[CommandDoc] = &[
    CommandDoc {
        name: "flux",
        summary: "Docker, container, host, and compose operations",
        usage: &[
            "synapse flux docker info|df|networks|volumes [--host H]",
            "synapse flux docker images [--host H] [--dangling-only]",
            "synapse flux docker pull --host H --image IMG",
            "synapse flux docker build --host H --context /abs/path --tag TAG [--dockerfile REL] [--no-cache]",
            "synapse flux docker rmi --host H --image IMG --force",
            "synapse flux docker prune --host H --target containers|images|volumes|networks|buildcache|all --force",
            "synapse flux container list [--host H] [--state S] [--name-filter N] [--image-filter I] [--label-filter K=V]",
            "synapse flux container inspect --container-id ID [--host H] [--summary]",
            "synapse flux container logs --container-id ID [--host H] [--lines N] [--since T] [--until T] [--grep S] [--stream stdout|stderr|both]",
            "synapse flux container stats [--container-id ID] [--host H]",
            "synapse flux container top --container-id ID [--host H]",
            "synapse flux container search --query Q [--host H]",
            "synapse flux host status|info|uptime|resources|network [--host HOST]",
            "synapse flux host services --host HOST [--state STATE] [--service NAME]",
            "synapse flux host mounts --host HOST",
            "synapse flux host ports --host HOST [--protocol tcp|udp] [--limit N] [--offset N]",
            "synapse flux host doctor --host HOST [--checks c1,c2,...]",
            "synapse flux compose list --host HOST",
            "synapse flux compose status|up|down|restart|recreate|logs|build|pull|refresh --host HOST --project P [--service SVC]",
            "All flux actions accept [--response-format markdown|json].",
        ],
    },
    CommandDoc {
        name: "scout",
        summary: "SSH filesystem, process, transfer, ZFS, and log operations",
        usage: &[
            "synapse scout nodes",
            "synapse scout peek --host HOST --path PATH [--tree] [--depth N]",
            "synapse scout find --host HOST --path PATH --pattern GLOB [--depth N] [--limit N]",
            "synapse scout ps --host HOST [--sort cpu|mem|pid|time] [--grep S] [--user U] [--limit N]",
            "synapse scout df --host HOST [--path PATH]",
            "synapse scout delta --source-host H --source-path P (--target-host H --target-path P | --content STR)",
            "synapse scout exec --host HOST --command CMD [--path PATH] [--args A1 A2...]",
            "synapse scout emit --command CMD --target HOST:PATH[,HOST:PATH...] [--timeout S]",
            "synapse scout beam --source-host H --source-path P --dest-host H --dest-path P",
            "synapse scout zfs pools|datasets|snapshots --host HOST [--pool POOL]",
            "synapse scout logs syslog|journal|dmesg|auth --host HOST [--lines N] [--grep STR]",
            "All scout actions accept [--response-format markdown|json].",
        ],
    },
    CommandDoc {
        name: "serve",
        summary: "Start the MCP HTTP server",
        usage: &["synapse serve", "synapse serve mcp"],
    },
    CommandDoc {
        name: "mcp",
        summary: "Start the MCP stdio transport",
        usage: &["synapse mcp"],
    },
    CommandDoc {
        name: "doctor",
        summary: "Run environment pre-flight checks",
        usage: &["synapse doctor [--json]"],
    },
    CommandDoc {
        name: "watch",
        summary: "Poll /health and emit state changes",
        usage: &["synapse watch [--url URL] [--interval N]"],
    },
    CommandDoc {
        name: "setup",
        summary: "Initialize, check, and repair plugin setup",
        usage: &[
            "synapse setup check",
            "synapse setup repair",
            "synapse setup install",
            "synapse setup plugin-hook [--no-repair]",
        ],
    },
    CommandDoc {
        name: "help",
        summary: "Show the action reference",
        usage: &["synapse help [--response-format markdown|json]"],
    },
];

const NESTED_CATALOG: &[NestedCommandDoc] = &[
    NestedCommandDoc {
        path: "flux docker",
        summary: "Docker engine operations",
        usage: &[
            "synapse flux docker info|df|networks|volumes [--host H]",
            "synapse flux docker images [--host H] [--dangling-only]",
            "synapse flux docker pull --host H --image IMG",
            "synapse flux docker build --host H --context /abs/path --tag TAG [--dockerfile REL] [--no-cache]",
            "synapse flux docker rmi --host H --image IMG --force",
            "synapse flux docker prune --host H --target containers|images|volumes|networks|buildcache|all --force",
        ],
    },
    NestedCommandDoc {
        path: "flux container",
        summary: "Container read operations",
        usage: &[
            "synapse flux container list [--host H] [--state S] [--name-filter N] [--image-filter I] [--label-filter K=V]",
            "synapse flux container inspect --container-id ID [--host H] [--summary]",
            "synapse flux container logs --container-id ID [--host H] [--lines N] [--since T] [--until T] [--grep S] [--stream stdout|stderr|both]",
            "synapse flux container stats [--container-id ID] [--host H]",
            "synapse flux container top --container-id ID [--host H]",
            "synapse flux container search --query Q [--host H]",
        ],
    },
    NestedCommandDoc {
        path: "flux host",
        summary: "Host status and inventory operations",
        usage: &[
            "synapse flux host status|info|uptime|resources|network [--host HOST]",
            "synapse flux host services --host HOST [--state STATE] [--service NAME]",
            "synapse flux host mounts --host HOST",
            "synapse flux host ports --host HOST [--protocol tcp|udp] [--limit N] [--offset N]",
            "synapse flux host doctor --host HOST [--checks c1,c2,...]",
        ],
    },
    NestedCommandDoc {
        path: "flux compose",
        summary: "Docker Compose project operations",
        usage: &[
            "synapse flux compose list --host HOST",
            "synapse flux compose status --host HOST --project P [--service SVC]",
            "synapse flux compose up|down|restart|recreate|logs|build|pull|refresh --host HOST --project P [--service SVC]",
        ],
    },
    NestedCommandDoc {
        path: "scout zfs",
        summary: "ZFS pool, dataset, and snapshot inspection",
        usage: &["synapse scout zfs pools|datasets|snapshots --host HOST [--pool POOL]"],
    },
    NestedCommandDoc {
        path: "scout logs",
        summary: "Remote syslog, journal, dmesg, and auth log reads",
        usage: &["synapse scout logs syslog|journal|dmesg|auth --host HOST [--lines N] [--grep STR]"],
    },
    NestedCommandDoc {
        path: "setup plugin-hook",
        summary: "Run plugin setup hook repair or audit mode",
        usage: &["synapse setup plugin-hook [--no-repair]"],
    },
];

const GLOBAL_OPTIONS: &[(&str, &str)] = &[
    ("-h, --help", "Display help (top-level or per-command)"),
    ("--version", "Print version and exit"),
    ("--color <when>", "Colorize output: always, never, or auto"),
    (
        "--no-color",
        "Disable colored output (alias for --color=never)",
    ),
];

const ENVIRONMENT: &[(&str, &str)] = &[
    ("SYNAPSE_HOSTS_CONFIG", "Host topology as a JSON array"),
    (
        "SYNAPSE_CONFIG_FILE",
        "Host config file path (falls back to ~/.ssh/config)",
    ),
    ("SYNAPSE_MCP_HOST", "Bind host (default 127.0.0.1)"),
    ("SYNAPSE_MCP_PORT", "Bind port (default 40080)"),
    ("SYNAPSE_MCP_NO_AUTH", "Disable auth (loopback only)"),
    ("SYNAPSE_MCP_TOKEN", "Static bearer token"),
    ("RUST_LOG", "Log filter; stdio logs always go to stderr"),
];

const QUICK_START: &[&str] = &[
    "synapse flux container list --host local",
    "synapse scout nodes",
    "synapse doctor",
];

const FLUX_USAGE_SECTIONS: &[UsageSection] = &[
    UsageSection {
        title: "Docker",
        lines: &[
            "synapse flux docker info|df|networks|volumes [--host H]",
            "synapse flux docker images [--host H] [--dangling-only]",
            "synapse flux docker pull --host H --image IMG",
            "synapse flux docker build --host H --context /abs/path --tag TAG [--dockerfile REL] [--no-cache]",
            "synapse flux docker rmi --host H --image IMG --force",
            "synapse flux docker prune --host H --target containers|images|volumes|networks|buildcache|all --force",
        ],
    },
    UsageSection {
        title: "Containers",
        lines: &[
            "synapse flux container list [--host H] [--state S] [--name-filter N] [--image-filter I] [--label-filter K=V]",
            "synapse flux container inspect --container-id ID [--host H] [--summary]",
            "synapse flux container logs --container-id ID [--host H] [--lines N] [--since T] [--until T] [--grep S] [--stream stdout|stderr|both]",
            "synapse flux container stats [--container-id ID] [--host H]",
            "synapse flux container top --container-id ID [--host H]",
            "synapse flux container search --query Q [--host H]",
        ],
    },
    UsageSection {
        title: "Host",
        lines: &[
            "synapse flux host status|info|uptime|resources|network [--host HOST]",
            "synapse flux host services --host HOST [--state STATE] [--service NAME]",
            "synapse flux host mounts --host HOST",
            "synapse flux host ports --host HOST [--protocol tcp|udp] [--limit N] [--offset N]",
            "synapse flux host doctor --host HOST [--checks c1,c2,...]",
        ],
    },
    UsageSection {
        title: "Compose",
        lines: &[
            "synapse flux compose list --host HOST",
            "synapse flux compose status --host HOST --project P [--service SVC]",
            "synapse flux compose up|down|restart|recreate|logs|build|pull|refresh --host HOST --project P [--service SVC]",
            "All flux actions accept [--response-format markdown|json].",
        ],
    },
];

const SCOUT_USAGE_SECTIONS: &[UsageSection] = &[
    UsageSection {
        title: "Inventory & Files",
        lines: &[
            "synapse scout nodes",
            "synapse scout peek --host HOST --path PATH [--tree] [--depth N]",
            "synapse scout find --host HOST --path PATH --pattern GLOB [--depth N] [--limit N]",
            "synapse scout df --host HOST [--path PATH]",
        ],
    },
    UsageSection {
        title: "Processes & Exec",
        lines: &[
            "synapse scout ps --host HOST [--sort cpu|mem|pid|time] [--grep S] [--user U] [--limit N]",
            "synapse scout exec --host HOST --command CMD [--path PATH] [--args A1 A2...]",
        ],
    },
    UsageSection {
        title: "Transfer",
        lines: &[
            "synapse scout delta --source-host H --source-path P (--target-host H --target-path P | --content STR)",
            "synapse scout emit --command CMD --target HOST:PATH[,HOST:PATH...] [--timeout S]",
            "synapse scout beam --source-host H --source-path P --dest-host H --dest-path P",
        ],
    },
    UsageSection {
        title: "ZFS & Logs",
        lines: &[
            "synapse scout zfs pools|datasets|snapshots --host HOST [--pool POOL]",
            "synapse scout logs syslog|journal|dmesg|auth --host HOST [--lines N] [--grep STR]",
            "All scout actions accept [--response-format markdown|json].",
        ],
    },
];

fn paint(color: bool, code: &str, text: &str) -> String {
    if color {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn heading(color: bool, text: &str) -> String {
    if color {
        format!("{BOLD}{CYAN_ANSI}{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn push_row(
    out: &mut String,
    color: bool,
    indent: usize,
    label_width: usize,
    label_code: &str,
    label: &str,
    desc: &str,
) {
    if label.chars().count() > label_width {
        out.push_str(&format!(
            "{:indent$}{}\n",
            "",
            paint(color, label_code, label),
            indent = indent
        ));
        out.push_str(&format!(
            "{:width$}{}\n",
            "",
            paint(color, PRIMARY_ANSI, desc),
            width = indent + label_width + 1
        ));
        return;
    }
    let padded = format!("{label:<label_width$}");
    out.push_str(&format!(
        "{:indent$}{} {}\n",
        "",
        paint(color, label_code, &padded),
        paint(color, PRIMARY_ANSI, desc),
        indent = indent
    ));
}

pub(crate) fn render_top_level(color: bool) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(&format!("  {}\n", heading(color, "SYNAPSE2 CLI")));
    out.push_str(&format!(
        "  {}\n",
        paint(color, CYAN_ANSI, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    ));
    out.push_str(&format!(
        "  Version {}  |  {}\n\n",
        env!("CARGO_PKG_VERSION"),
        paint(color, PRIMARY_ANSI, TAGLINE)
    ));

    out.push_str(&format!("  {}\n", heading(color, "Usage")));
    out.push_str(&format!(
        "  {}\n\n",
        paint(color, PRIMARY_ANSI, "synapse [options] <command> [args]")
    ));

    out.push_str(&format!("  {}\n", heading(color, "Quick Start")));
    for example in QUICK_START {
        out.push_str(&format!("  {}\n", paint(color, PRIMARY_ANSI, example)));
    }
    out.push('\n');

    out.push_str(&format!("  {}\n", heading(color, "Global Options")));
    for (flag, desc) in GLOBAL_OPTIONS {
        push_row(&mut out, color, 2, 28, PRIMARY_ANSI, flag, desc);
    }
    out.push('\n');

    out.push_str(&format!("  {}\n", heading(color, "Environment")));
    for (name, desc) in ENVIRONMENT {
        push_row(&mut out, color, 2, 28, PRIMARY_ANSI, name, desc);
    }
    out.push('\n');

    out.push_str(&format!("  {}\n", heading(color, "Commands")));
    for (section, names) in SECTIONS {
        out.push_str(&format!("  {}\n", paint(color, CYAN_ANSI, section)));
        for name in *names {
            if let Some(doc) = lookup(name) {
                push_row(&mut out, color, 4, 18, PRIMARY_ANSI, doc.name, doc.summary);
            }
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "  {}\n",
        paint(
            color,
            PRIMARY_ANSI,
            "→ Run synapse <command> --help for command-specific flags"
        )
    ));
    out
}

pub(crate) fn render_command(name: &str, color: bool) -> Option<String> {
    if name == "flux" {
        return Some(render_grouped_doc(
            "flux",
            "Docker, container, host, and compose operations",
            FLUX_USAGE_SECTIONS,
            color,
        ));
    }
    if name == "scout" {
        return Some(render_grouped_doc(
            "scout",
            "SSH filesystem, process, transfer, ZFS, and log operations",
            SCOUT_USAGE_SECTIONS,
            color,
        ));
    }
    if let Some(doc) = nested_lookup(name) {
        return Some(render_doc(doc.path, doc.summary, doc.usage, color));
    }
    let doc = lookup(name)?;
    Some(render_doc(doc.name, doc.summary, doc.usage, color))
}

fn render_doc(name: &str, summary: &str, usage: &[&str], color: bool) -> String {
    let mut out = String::with_capacity(512);
    out.push_str(&format!(
        "  {}  {}\n\n",
        heading(color, name),
        paint(color, PRIMARY_ANSI, summary)
    ));
    out.push_str(&format!("  {}\n", heading(color, "Usage")));
    for line in usage {
        out.push_str(&format!("  {}\n", paint(color, PRIMARY_ANSI, line)));
    }
    out
}

fn render_grouped_doc(name: &str, summary: &str, sections: &[UsageSection], color: bool) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str(&format!(
        "  {}  {}\n\n",
        heading(color, name),
        paint(color, PRIMARY_ANSI, summary)
    ));
    out.push_str(&format!("  {}\n", heading(color, "Usage")));
    for section in sections {
        out.push_str(&format!("  {}\n", heading(color, section.title)));
        for line in section.lines {
            out.push_str(&format!("  {}\n", paint(color, PRIMARY_ANSI, line)));
        }
        out.push('\n');
    }
    out
}

fn lookup(name: &str) -> Option<&'static CommandDoc> {
    CATALOG.iter().find(|doc| doc.name == name)
}

fn nested_lookup(path: &str) -> Option<&'static NestedCommandDoc> {
    NESTED_CATALOG.iter().find(|doc| doc.path == path)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HelpRequest {
    TopLevel,
    Command(String),
    None,
}

pub(crate) fn classify_help(args: &[String]) -> HelpRequest {
    let scan: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .take_while(|arg| *arg != "--")
        .collect();
    if scan.is_empty() {
        return HelpRequest::None;
    }
    let has_help =
        scan.iter().any(|arg| matches!(*arg, "-h" | "--help")) || scan.first() == Some(&"help");
    if !has_help {
        return HelpRequest::None;
    }

    let positionals: Vec<&str> = scan
        .into_iter()
        .filter(|arg| !arg.starts_with('-') && *arg != "help")
        .collect();
    if positionals.len() >= 2 {
        let nested = format!("{} {}", positionals[0], positionals[1]);
        if nested_lookup(&nested).is_some() {
            return HelpRequest::Command(nested);
        }
    }
    match positionals.first().copied() {
        Some(name) if lookup(name).is_some() => HelpRequest::Command(name.to_string()),
        _ => HelpRequest::TopLevel,
    }
}

pub(crate) fn maybe_handle_help(args: &[String]) -> bool {
    match classify_help(args) {
        HelpRequest::TopLevel => {
            print!("{}", render_top_level(super::color::color_enabled()));
            true
        }
        HelpRequest::Command(name) => {
            if let Some(rendered) = render_command(&name, super::color::color_enabled()) {
                print!("{rendered}");
            } else {
                print!("{}", render_top_level(super::color::color_enabled()));
            }
            true
        }
        HelpRequest::None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_help_matches_cortex_grouped_shape() {
        let rendered = render_top_level(false);
        assert!(rendered.contains("  SYNAPSE2 CLI\n"));
        assert!(rendered.contains("  Version "));
        assert!(rendered.contains("  Usage\n"));
        assert!(rendered.contains("  synapse [options] <command> [args]\n"));
        assert!(rendered.contains("  Quick Start\n"));
        assert!(rendered.contains("  Global Options\n"));
        assert!(rendered.contains("  --color <when>"));
        assert!(rendered.contains("  Environment\n"));
        assert!(rendered.contains("  Commands\n"));
        assert!(rendered.contains("  Flux\n"));
        assert!(rendered.contains("    flux"));
        assert!(rendered.contains("  Scout\n"));
        assert!(rendered.contains("    scout"));
        assert!(rendered.contains("  Runtime & Setup\n"));
        assert!(rendered.contains("→ Run synapse <command> --help for command-specific flags"));
    }

    #[test]
    fn colorized_help_uses_cortex_palette_codes() {
        let rendered = render_top_level(true);
        assert!(rendered.contains(CYAN_ANSI));
        assert!(rendered.contains(PRIMARY_ANSI));
        assert!(rendered.contains(&format!(
            "  {PRIMARY_ANSI}synapse flux container list --host local{RESET}\n"
        )));
        assert!(rendered.contains(&format!(
            "  {PRIMARY_ANSI}--color <when>              {RESET}"
        )));
        assert!(rendered.contains(&format!(
            "  {PRIMARY_ANSI}SYNAPSE_HOSTS_CONFIG        {RESET}"
        )));
        assert!(!rendered.contains(&format!("{CYAN_ANSI}synapse [options]")));
    }

    #[test]
    fn command_help_renders_usage_only_for_that_command() {
        let rendered = render_command("flux container", false).unwrap();
        assert!(rendered.contains("  flux container  Container read operations"));
        assert!(rendered.contains("synapse flux container list"));
        assert!(!rendered.contains("Quick Start"));
    }

    #[test]
    fn top_level_flux_help_groups_usage_and_uses_white_content() {
        let rendered = render_command("flux", true).unwrap();
        assert!(rendered.contains(&format!("{CYAN_ANSI}Docker{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}Containers{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}Host{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}Compose{RESET}")));
        assert!(rendered.contains(&format!(
            "{PRIMARY_ANSI}synapse flux docker images [--host H] [--dangling-only]{RESET}"
        )));
        assert!(!rendered.contains(&format!("{CYAN_ANSI}synapse flux docker images")));
    }

    #[test]
    fn top_level_scout_help_groups_usage_and_uses_white_content() {
        let rendered = render_command("scout", true).unwrap();
        assert!(rendered.contains(&format!("{CYAN_ANSI}Inventory & Files{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}Processes & Exec{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}Transfer{RESET}")));
        assert!(rendered.contains(&format!("{CYAN_ANSI}ZFS & Logs{RESET}")));
        assert!(rendered.contains(&format!("{PRIMARY_ANSI}synapse scout nodes{RESET}")));
        assert!(!rendered.contains(&format!("{CYAN_ANSI}synapse scout nodes")));
    }

    #[test]
    fn help_classification_is_positional() {
        assert_eq!(
            classify_help(&["flux".into(), "container".into(), "--help".into()]),
            HelpRequest::Command("flux container".into())
        );
        assert_eq!(classify_help(&["help".into()]), HelpRequest::TopLevel);
        assert_eq!(
            classify_help(&[
                "scout".into(),
                "find".into(),
                "--pattern".into(),
                "help".into()
            ]),
            HelpRequest::None
        );
    }
}
