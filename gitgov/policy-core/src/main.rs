use gitgov_policy_core::{validate_git_policy_change, PolicyGitValidationStatus};
use std::env;
use std::path::PathBuf;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("validate") {
        print_usage_and_exit(2);
    }

    let mut repo = PathBuf::from(".");
    let mut base_ref = None;
    let mut head_ref = None;
    let mut blocking = false;
    let mut json = false;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                repo = PathBuf::from(args.get(index).unwrap_or_else(|| {
                    eprintln!("--repo requires a path");
                    std::process::exit(2);
                }));
            }
            "--base-ref" => {
                index += 1;
                base_ref = Some(args.get(index).cloned().unwrap_or_else(|| {
                    eprintln!("--base-ref requires a git ref");
                    std::process::exit(2);
                }));
            }
            "--head-ref" => {
                index += 1;
                head_ref = Some(args.get(index).cloned().unwrap_or_else(|| {
                    eprintln!("--head-ref requires a git ref");
                    std::process::exit(2);
                }));
            }
            "--blocking" => blocking = true,
            "--json" => json = true,
            _ => print_usage_and_exit(2),
        }
        index += 1;
    }

    let Some(base_ref) = base_ref else {
        print_usage_and_exit(2);
    };
    let Some(head_ref) = head_ref else {
        print_usage_and_exit(2);
    };

    let result = validate_git_policy_change(repo, &base_ref, &head_ref, blocking);

    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Policy-as-Code validation: {:?}", result.status);
        for path in &result.changed_policy_paths {
            println!("changed policy: {}", path);
        }
        for change in &result.changes {
            println!(
                "- {:?}: {} ({:?} -> {:?})",
                change.severity, change.message, change.before, change.after
            );
        }
        for error in &result.errors {
            eprintln!("error: {}", error);
        }
    }

    let exit_code = match result.status {
        PolicyGitValidationStatus::Invalid => 1,
        _ if !result.allowed => 1,
        _ => 0,
    };
    std::process::exit(exit_code);
}

fn print_usage_and_exit(code: i32) -> ! {
    eprintln!(
        "usage: gitgov-policy validate --repo <path> --base-ref <ref> --head-ref <ref> [--blocking] [--json]"
    );
    std::process::exit(code);
}
