use super::{parse_args_from, Command};

#[test]
fn parses_flux_and_scout_commands() {
    match parse_args_from(["flux", "docker", "images"]).unwrap() {
        Some(Command::FluxDocker(args)) => assert_eq!(args.subaction, "images"),
        other => panic!("expected FluxDocker, got {other:?}"),
    }
    match parse_args_from(["flux", "host", "status", "--host", "local"]).unwrap() {
        Some(Command::FluxHost(args)) => {
            assert_eq!(args.subaction, "status");
            assert_eq!(args.host.as_deref(), Some("local"));
        }
        other => panic!("expected FluxHost, got {other:?}"),
    }
    assert_eq!(
        parse_args_from(["scout", "nodes"]).unwrap(),
        Some(Command::ScoutNodes)
    );
}

#[test]
fn rejects_malformed_synapse_commands() {
    assert!(parse_args_from(["flux"]).is_err());
    assert!(parse_args_from(["scout", "peek", "--host", "local"]).is_err());
    assert!(parse_args_from(["scout", "exec", "--host", "local"]).is_err());
}
