pub fn parse_arguments(args: &[String]) -> (bool, bool, bool, bool, Option<String>) {
    let mut follow_slink = false;
    let mut prohibit_mount_crossing = false;
    let mut prohibit_non_subdir_transition = false;
    let mut prevent_multiple_pathnames = false;
    let mut directory_path = None;

    if args.len() < 2 {
        eprintln!("Error: Insufficient arguments");
        return (follow_slink, prohibit_mount_crossing, prohibit_non_subdir_transition, prevent_multiple_pathnames, directory_path);
    }

    for arg in args.iter().take(args.len() - 1).skip(1) {
        match arg.as_str() {
            "-L" => follow_slink = true,
            "-M" => prohibit_mount_crossing = true,
            "-B" => prohibit_non_subdir_transition = true,
            "-O" => prevent_multiple_pathnames = true,
            _ => eprintln!("Warning: Unrecognized option {}", arg),
        }
    }

    directory_path = args.last().cloned();

    (follow_slink, prohibit_mount_crossing, prohibit_non_subdir_transition, prevent_multiple_pathnames, directory_path)
}
