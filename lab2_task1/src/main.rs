mod exit_err;
mod field_ptr;
mod offset_of;
mod parse_arguments;
mod str_to_cstring;
mod s_isdir;
use core::ffi::CStr;
use field_ptr::field_ptr;
use libc;
use std::env;
use std::mem;
use str_to_cstring::str_to_cstring;
use s_isdir::s_isdir;
use std::collections::VecDeque;

fn get_cwd(fd_start: libc::c_int) -> String {
    let mut path_list = VecDeque::new();

    let mut fd = fd_start.clone();
    let mut cfd = fd_start.clone();
    let mut statbuf: libc::stat = unsafe { mem::zeroed() };
    let mut dirent: libc::dirent;
    let mut dir: *mut libc::DIR;
    let mut ent: *mut libc::dirent;

    unsafe {
        if (libc::stat(str_to_cstring("/").as_ptr(), &mut statbuf as _)) != 0 {
            exit_err!("stat");
        }
        let root_dev: libc::dev_t = statbuf.st_dev;
        let root_ino: libc::ino_t = statbuf.st_ino;
        let mut dev: libc::dev_t = statbuf.st_dev;
        let mut ino: libc::ino_t = statbuf.st_ino;
        let mut d_name_var: &str;
        loop {
            if libc::fstatat(cfd, str_to_cstring(".").as_ptr(), &mut statbuf as _, 0) != 0 {
                exit_err!("fstatat");
            }
            dev = statbuf.st_dev;
            ino = statbuf.st_ino;

            if root_ino == ino && root_dev == dev {
                if cfd != fd_start {
                    if libc::close(cfd) != 0 {
                        exit_err!("fstatat");
                    }
                    break;
                }
            }

            fd = libc::openat(cfd, str_to_cstring("..").as_ptr(), libc::O_RDONLY, 0);
            if fd < 0 {
                exit_err!("openat cfd:{cfd} {fd}");
            }
            dir = libc::fdopendir(fd);

            if (dir.is_null()) {
                exit_err!("fdopendir");
            }

            loop {
                *libc::__errno_location() = 0;
                ent = libc::readdir(dir);
                if (ent.is_null()) {
                    if *libc::__errno_location() != 0 {
                        exit_err!("readdir()");
                    }
                    break;
                }
                d_name_var = CStr::from_ptr(field_ptr!(ent, libc::dirent, d_name).cast())
                    .to_str()
                    .unwrap();
                
                if d_name_var.starts_with('.') && (d_name_var == "." || d_name_var == "..") {
                    continue;
                }
                if (libc::fstatat(
                    fd,
                    str_to_cstring(d_name_var).as_ptr(),
                    &mut statbuf as _,
                    libc::AT_SYMLINK_NOFOLLOW,
                ) < 0)
                {
                    if (*libc::__errno_location() == libc::ENOENT) {
                        continue;
                    }
                    exit_err!("fstatat()");
                }
                if s_isdir(statbuf.st_mode) &&
                    statbuf.st_ino == ino && statbuf.st_dev == dev {
                        path_list.push_front(format!("{d_name_var}"));
                    break;
                }
            }

            if libc::closedir(dir) < 0 {
                exit_err!("closedir");
            }
            if ent.is_null() {
                exit_err!("ent is null");
            }
            fd = cfd;
            cfd = libc::openat(cfd, str_to_cstring("..").as_ptr(), libc::O_RDONLY);
            if cfd < 0 {
                exit_err!("openat");
            }
            if (fd != fd_start) {
                if (libc::close(fd) < 0) {
                    exit_err!("openat");
                }
            }
        }
    }
    let mut cwd: String = String::new();

    for path in &path_list {
        cwd.push_str(path);
        cwd.push_str("/");
    }
    cwd
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (root_dir, work_dir) = parse_arguments::parse_arguments(&args);
    match root_dir {
        Some(root) => {
            unsafe {
                if libc::chroot(str_to_cstring(&root).as_ptr()) < 0 {
                    exit_err!("chroot");
                }
            }
        }
        None => {}
    }

    match work_dir {
        Some(work) => {
            unsafe {
                if libc::chdir(str_to_cstring(&work).as_ptr()) < 0 {
                    exit_err!("chdir");
                }
            }
        }
        None => {}
    }
    let cwd = get_cwd(libc::AT_FDCWD);

    println!("{cwd}");
}
