#[macro_export]
macro_rules! exit_err {
    () => {{
        compile_error!("specify first argument");
    }};
    ($($arg:tt)*) => {{
        use std::io::{self, Write};

        let error = io::Error::last_os_error();
        let _ = io::stdout().flush();

        eprint!("FAIL:2 ");
        eprint!($($arg)*);
        eprintln!(": {}", error);

        std::process::exit(1);
    }};
}
