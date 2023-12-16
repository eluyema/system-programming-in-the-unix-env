use libc::mode_t;

const S_IFMT: mode_t = 0o170000;
const S_IFDIR: mode_t = 0o040000;
pub fn s_isdir(mode: mode_t) -> bool {
    mode & S_IFMT == S_IFDIR
}
