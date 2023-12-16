pub fn parse_arguments(args: &[String]) -> (Option<String>, Option<String>) {
    let mut root_dir: Option<String> = None;
    let mut work_dir: Option<String> = None;

    let mut args_iter = args.iter().skip(1);

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--root" => {
                root_dir = args_iter.next().cloned();
            },
            "--work" => {
                work_dir = args_iter.next()
                              .and_then(|a| a.parse().ok());
            },
            _ => eprintln!("Warning: Unrecognized option {}", arg),
        }
    }

    (root_dir, work_dir)
}