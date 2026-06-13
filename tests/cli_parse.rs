use synapse2::cli::{parse_args_from, Command, SetupCommand};

#[test]
fn flux_docker_info_parsed() {
    let cmd = parse_args_from(["flux", "docker", "info"]).unwrap();
    match cmd {
        Some(Command::FluxDocker(args)) => assert_eq!(args.subaction, "info"),
        other => panic!("expected FluxDocker, got {other:?}"),
    }
}

#[test]
fn flux_docker_flags_and_values_parse() {
    let cmd = parse_args_from([
        "flux",
        "docker",
        "build",
        "--host",
        "dookie",
        "--context",
        "/srv/app",
        "--tag",
        "app:test",
        "--dockerfile",
        "Dockerfile.dev",
        "--no-cache",
        "--response-format",
        "json",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxDocker(args)) => {
            assert_eq!(args.subaction, "build");
            assert_eq!(args.host.as_deref(), Some("dookie"));
            assert_eq!(args.context.as_deref(), Some("/srv/app"));
            assert_eq!(args.tag.as_deref(), Some("app:test"));
            assert_eq!(args.dockerfile.as_deref(), Some("Dockerfile.dev"));
            assert_eq!(args.no_cache, Some(true));
            assert_eq!(args.response_format.as_deref(), Some("json"));
        }
        other => panic!("expected FluxDocker, got {other:?}"),
    }

    let cmd = parse_args_from([
        "flux", "docker", "prune", "--host", "dookie", "--target", "images", "--force",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxDocker(args)) => {
            assert_eq!(args.subaction, "prune");
            assert_eq!(args.prune_target.as_deref(), Some("images"));
            assert_eq!(args.force, Some(true));
        }
        other => panic!("expected FluxDocker, got {other:?}"),
    }

    let cmd = parse_args_from(["flux", "docker", "images", "--dangling-only"]).unwrap();
    match cmd {
        Some(Command::FluxDocker(args)) => assert_eq!(args.dangling_only, Some(true)),
        other => panic!("expected FluxDocker, got {other:?}"),
    }
}

#[test]
fn flux_container_logs_parsed() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "logs",
        "--container-id",
        "abc",
        "--lines",
        "20",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "logs");
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.lines, Some(20));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_list_filters_parse() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "list",
        "--host",
        "dookie",
        "--state",
        "running",
        "--name-filter",
        "nginx",
        "--image-filter",
        "nginx",
        "--label-filter",
        "tier=edge",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "list");
            assert_eq!(args.host.as_deref(), Some("dookie"));
            assert_eq!(args.state.as_deref(), Some("running"));
            assert_eq!(args.name_filter.as_deref(), Some("nginx"));
            assert_eq!(args.image_filter.as_deref(), Some("nginx"));
            assert_eq!(args.label_filter.as_deref(), Some("tier=edge"));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_inspect_summary_flag_parses() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "inspect",
        "--container-id",
        "abc",
        "--summary",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "inspect");
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.summary, Some(true), "--summary must set summary=true");
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_rejects_invalid_response_format() {
    let err =
        parse_args_from(["flux", "container", "list", "--response-format", "xml"]).unwrap_err();
    assert!(
        err.to_string().contains("response_format")
            || err.to_string().to_lowercase().contains("xml")
            || err.to_string().contains("markdown"),
        "got: {err}"
    );
}

#[test]
fn container_accepts_valid_response_format() {
    let cmd = parse_args_from(["flux", "container", "list", "--response-format", "json"]).unwrap();
    assert!(matches!(cmd, Some(Command::FluxContainer(_))));
}

#[test]
fn container_search_query_parses() {
    let cmd = parse_args_from(["flux", "container", "search", "--query", "web"]).unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "search");
            assert_eq!(args.query.as_deref(), Some("web"));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_logs_filters_parse() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "logs",
        "--container-id",
        "abc",
        "--since",
        "30m",
        "--until",
        "now",
        "--grep",
        "ERROR",
        "--stream",
        "stderr",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.since.as_deref(), Some("30m"));
            assert_eq!(args.until.as_deref(), Some("now"));
            assert_eq!(args.grep.as_deref(), Some("ERROR"));
            assert_eq!(args.stream.as_deref(), Some("stderr"));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_lifecycle_and_recreate_flags_parse() {
    let cmd = parse_args_from(["flux", "container", "restart", "--container-id", "abc"]).unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "restart");
            assert_eq!(args.container_id.as_deref(), Some("abc"));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }

    let cmd = parse_args_from([
        "flux",
        "container",
        "recreate",
        "--container-id",
        "abc",
        "--no-pull",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "recreate");
            assert_eq!(args.pull, Some(false));
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_exec_command_accepts_flags_after_command() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "exec",
        "--container-id",
        "abc",
        "--command",
        "sh",
        "-c",
        "echo ok",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.subaction, "exec");
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.command, ["sh", "-c", "echo ok"]);
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_exec_command_accepts_double_dash_flag_after_command() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "exec",
        "--container-id",
        "abc",
        "--command",
        "tool",
        "--flag",
        "value",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.command, ["tool", "--flag", "value"]);
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_exec_options_before_command_are_parsed_as_synapse_options() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "exec",
        "--host",
        "dookie",
        "--container-id",
        "abc",
        "--timeout",
        "5000",
        "--command",
        "printenv",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.host.as_deref(), Some("dookie"));
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.exec_timeout_ms, Some(5000));
            assert_eq!(args.command, ["printenv"]);
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_exec_user_and_workdir_parse() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "exec",
        "--container-id",
        "abc",
        "--user",
        "root",
        "--workdir",
        "/app",
        "--command",
        "pwd",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.exec_user.as_deref(), Some("root"));
            assert_eq!(args.exec_workdir.as_deref(), Some("/app"));
            assert_eq!(args.command, ["pwd"]);
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn container_exec_synapse_options_after_command_are_container_argv() {
    let cmd = parse_args_from([
        "flux",
        "container",
        "exec",
        "--container-id",
        "abc",
        "--command",
        "tool",
        "--timeout",
        "5000",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxContainer(args)) => {
            assert_eq!(args.container_id.as_deref(), Some("abc"));
            assert_eq!(args.exec_timeout_ms, None);
            assert_eq!(args.command, ["tool", "--timeout", "5000"]);
        }
        other => panic!("expected FluxContainer, got {other:?}"),
    }
}

#[test]
fn scout_commands_parse() {
    assert_eq!(
        parse_args_from(["scout", "nodes"]).unwrap(),
        Some(Command::ScoutNodes {
            response_format: None
        })
    );
    // peek with defaults
    assert_eq!(
        parse_args_from(["scout", "peek", "--host", "local", "--path", "/tmp"]).unwrap(),
        Some(Command::ScoutPeek {
            response_format: None,
            host: "local".into(),
            path: "/tmp".into(),
            tree: false,
            depth: 3,
        })
    );
    // exec with new boxed args shape
    let cmd = parse_args_from([
        "scout",
        "exec",
        "--host",
        "local",
        "--path",
        "/tmp",
        "--command",
        "ls",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutExec(a)) => {
            assert_eq!(a.host, "local");
            assert_eq!(a.path.as_deref(), Some("/tmp"));
            assert_eq!(a.command, "ls");
        }
        other => panic!("expected ScoutExec, got {other:?}"),
    }
    // find
    let cmd = parse_args_from([
        "scout",
        "find",
        "--host",
        "local",
        "--path",
        "/etc",
        "--pattern",
        "*.conf",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutFind(a)) => {
            assert_eq!(a.host, "local");
            assert_eq!(a.pattern, "*.conf");
        }
        other => panic!("expected ScoutFind, got {other:?}"),
    }
}

#[test]
fn flux_host_commands_parse_options() {
    let cmd = parse_args_from([
        "flux",
        "host",
        "ports",
        "--host",
        "dookie",
        "--protocol",
        "tcp",
        "--limit",
        "25",
        "--offset",
        "50",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxHost(args)) => {
            assert_eq!(args.subaction, "ports");
            assert_eq!(args.host.as_deref(), Some("dookie"));
            assert_eq!(args.protocol.as_deref(), Some("tcp"));
            assert_eq!(args.limit, Some(25));
            assert_eq!(args.offset, Some(50));
        }
        other => panic!("expected FluxHost, got {other:?}"),
    }

    let cmd = parse_args_from([
        "flux",
        "host",
        "doctor",
        "--host",
        "dookie",
        "--checks",
        "docker,logs",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxHost(args)) => {
            assert_eq!(args.subaction, "doctor");
            assert_eq!(args.checks.as_deref(), Some("docker,logs"));
        }
        other => panic!("expected FluxHost, got {other:?}"),
    }
}

#[test]
fn flux_compose_commands_parse_options() {
    let cmd = parse_args_from([
        "flux",
        "compose",
        "down",
        "--host",
        "tootie",
        "--project",
        "media",
        "--remove-volumes",
        "--force",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxCompose(args)) => {
            assert_eq!(args.subaction, "down");
            assert_eq!(args.host.as_deref(), Some("tootie"));
            assert_eq!(args.project.as_deref(), Some("media"));
            assert_eq!(args.remove_volumes, Some(true));
            assert_eq!(args.force, Some(true));
        }
        other => panic!("expected FluxCompose, got {other:?}"),
    }

    let cmd = parse_args_from([
        "flux",
        "compose",
        "logs",
        "--host",
        "tootie",
        "--project",
        "media",
        "--service",
        "plex",
        "--lines",
        "150",
        "--since",
        "1h",
    ])
    .unwrap();
    match cmd {
        Some(Command::FluxCompose(args)) => {
            assert_eq!(args.subaction, "logs");
            assert_eq!(args.service.as_deref(), Some("plex"));
            assert_eq!(args.lines, Some(150));
            assert_eq!(args.since.as_deref(), Some("1h"));
        }
        other => panic!("expected FluxCompose, got {other:?}"),
    }
}

#[test]
fn scout_remaining_commands_parse() {
    let cmd = parse_args_from([
        "scout", "ps", "--host", "dookie", "--sort", "mem", "--grep", "nginx", "--user", "root",
        "--limit", "10",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutPs(args)) => {
            assert_eq!(args.host, "dookie");
            assert_eq!(args.sort.as_deref(), Some("mem"));
            assert_eq!(args.grep.as_deref(), Some("nginx"));
            assert_eq!(args.user.as_deref(), Some("root"));
            assert_eq!(args.limit, Some(10));
        }
        other => panic!("expected ScoutPs, got {other:?}"),
    }

    let cmd = parse_args_from(["scout", "df", "--host", "dookie", "--path", "/srv"]).unwrap();
    match cmd {
        Some(Command::ScoutDf { host, path, .. }) => {
            assert_eq!(host, "dookie");
            assert_eq!(path.as_deref(), Some("/srv"));
        }
        other => panic!("expected ScoutDf, got {other:?}"),
    }

    let cmd = parse_args_from([
        "scout",
        "delta",
        "--source-host",
        "a",
        "--source-path",
        "/etc/hosts",
        "--target-host",
        "b",
        "--target-path",
        "/etc/hosts",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutDelta(args)) => {
            assert_eq!(args.source_host, "a");
            assert_eq!(args.target_host.as_deref(), Some("b"));
            assert_eq!(args.target_path.as_deref(), Some("/etc/hosts"));
        }
        other => panic!("expected ScoutDelta, got {other:?}"),
    }

    let cmd = parse_args_from([
        "scout",
        "emit",
        "--target",
        "a:/srv,b",
        "--command",
        "hostname",
        "--timeout",
        "5",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutEmit(args)) => {
            assert_eq!(args.targets.len(), 2);
            assert_eq!(args.targets[0].host, "a");
            assert_eq!(args.targets[0].path.as_deref(), Some("/srv"));
            assert_eq!(args.targets[1].host, "b");
            assert_eq!(args.targets[1].path, None);
            assert_eq!(args.timeout_secs, Some(5));
        }
        other => panic!("expected ScoutEmit, got {other:?}"),
    }

    let cmd = parse_args_from([
        "scout",
        "beam",
        "--source-host",
        "a",
        "--source-path",
        "/tmp/a",
        "--dest-host",
        "b",
        "--dest-path",
        "/tmp/b",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutBeam(args)) => {
            assert_eq!(args.source_host, "a");
            assert_eq!(args.dest_host, "b");
            assert_eq!(args.dest_path, "/tmp/b");
        }
        other => panic!("expected ScoutBeam, got {other:?}"),
    }
}

#[test]
fn scout_logs_and_zfs_variants_parse() {
    let cmd = parse_args_from([
        "scout",
        "logs",
        "journal",
        "--host",
        "dookie",
        "--lines",
        "200",
        "--grep",
        "failed",
        "--unit",
        "docker.service",
        "--priority",
        "err",
        "--since",
        "-1h",
        "--until",
        "now",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutLogs(args)) => {
            assert_eq!(args.subaction, "journal");
            assert_eq!(args.lines, 200);
            assert_eq!(args.grep.as_deref(), Some("failed"));
            assert_eq!(args.unit.as_deref(), Some("docker.service"));
            assert_eq!(args.priority.as_deref(), Some("err"));
            assert_eq!(args.since.as_deref(), Some("-1h"));
            assert_eq!(args.until.as_deref(), Some("now"));
        }
        other => panic!("expected ScoutLogs, got {other:?}"),
    }

    for subaction in ["syslog", "dmesg", "auth"] {
        let cmd = parse_args_from(["scout", "logs", subaction, "--host", "dookie"]).unwrap();
        match cmd {
            Some(Command::ScoutLogs(args)) => assert_eq!(args.subaction, subaction),
            other => panic!("expected ScoutLogs, got {other:?}"),
        }
    }

    let cmd = parse_args_from([
        "scout", "zfs", "pools", "--host", "dookie", "--pool", "tank",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutZfs(args)) => {
            assert_eq!(args.subaction, "pools");
            assert_eq!(args.pool.as_deref(), Some("tank"));
        }
        other => panic!("expected ScoutZfs, got {other:?}"),
    }

    let cmd = parse_args_from([
        "scout",
        "zfs",
        "snapshots",
        "--host",
        "dookie",
        "--dataset",
        "tank/data",
        "--limit",
        "25",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutZfs(args)) => {
            assert_eq!(args.subaction, "snapshots");
            assert_eq!(args.dataset.as_deref(), Some("tank/data"));
            assert_eq!(args.limit, Some(25));
        }
        other => panic!("expected ScoutZfs, got {other:?}"),
    }
}

#[test]
fn scout_zfs_recursive_flag_parses_without_value() {
    let cmd = parse_args_from([
        "scout",
        "zfs",
        "datasets",
        "--host",
        "local",
        "--pool",
        "tank",
        "--type",
        "filesystem",
        "--recursive",
    ])
    .unwrap();
    match cmd {
        Some(Command::ScoutZfs(args)) => {
            assert_eq!(args.subaction, "datasets");
            assert_eq!(args.host, "local");
            assert_eq!(args.pool.as_deref(), Some("tank"));
            assert_eq!(args.dataset_type.as_deref(), Some("filesystem"));
            assert!(args.recursive);
        }
        other => panic!("expected ScoutZfs, got {other:?}"),
    }
}

#[test]
fn setup_and_doctor_still_parse() {
    assert_eq!(
        parse_args_from(["setup", "plugin-hook", "--no-repair"]).unwrap(),
        Some(Command::Setup(SetupCommand::PluginHook { no_repair: true }))
    );
    assert_eq!(
        parse_args_from(["doctor", "--json"]).unwrap(),
        Some(Command::Doctor { json: true })
    );
}

#[test]
fn malformed_args_are_rejected() {
    for args in [
        &["flux", "container", "logs", "--container-id"][..],
        &["scout", "exec", "--host", "local", "--path", "/tmp"],
        &["watch", "--interval", "0"],
        &["setup", "plugin-hook", "--no-reapir"],
    ] {
        assert!(
            parse_args_from(args.iter().copied()).is_err(),
            "{args:?} should be rejected"
        );
    }
}
