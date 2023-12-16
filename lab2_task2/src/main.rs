use std::collections::LinkedList;
mod parse_arguments;
mod field_ptr;
mod offset_of;
mod exit_err;
mod s_isdir;
use std::env;
use core::ffi::CStr;
use field_ptr::field_ptr;
mod str_to_cstring;
use std::mem;
use std::collections::VecDeque;
use libc;
use str_to_cstring::str_to_cstring;
use std::rc::Rc;
use std::collections::HashSet;
use s_isdir::s_isdir;
use std::cell::RefCell;

#[derive(Clone)]
struct Options {
    follow_slink: bool,
    prevent_multiple_pathnames: bool,
}

impl Options {
    fn new(follow_slink: bool,
        prevent_multiple_pathnames: bool,) -> Options {
            Options {
                follow_slink, prevent_multiple_pathnames
            }
    }
}

#[derive(Clone)]
struct DirEntry {
    name: String,
    parent: Option<Rc<RefCell<DirEntry>>>,
    entries: LinkedList<Rc<RefCell<DirEntry>>>,
    dev: libc::dev_t,
    ino: libc::ino_t,
    loop_found: bool,
    cycle: bool,
    open_mode: i32,
    options: Options,
}


impl DirEntry {
    fn new(name: &str, options: Options, parent: Option<Rc<RefCell<DirEntry>>>) -> Rc<RefCell<DirEntry>>{
        let mut open_mode = libc::O_RDONLY | libc::O_DIRECTORY;
        if (!options.follow_slink) {
            open_mode |= libc::O_NOFOLLOW;
        }
        
        Rc::new(RefCell::new(DirEntry {
            name: name.to_string(),
            parent,
            entries: LinkedList::new(),
            dev: 0,
            ino: 0,
            loop_found: false,
            cycle: false,
            open_mode,
            options
        }))
    }

    fn fd(&self) -> i32 {
        let mut path: VecDeque<DirEntry> = VecDeque::new();
        let mut fd1: i32;
        let mut fd2: i32;

        let mut entry = Some(Rc::new(RefCell::new(self.clone())));
        while let Some(entry_ref) = entry {
            let e = entry_ref.borrow();
            path.push_front(entry_ref.borrow().clone());
            entry = e.parent.clone();
        }

        fd1 = libc::AT_FDCWD;

        unsafe {
            for en in path {

                let open_mode_masked: i32 = self.open_mode & !(if fd1 == libc::AT_FDCWD { libc::O_NOFOLLOW as i32 } else { 0 });

                fd2 = libc::openat(fd1, str_to_cstring(&en.name).as_ptr(),open_mode_masked);

                if (fd2 < 0) {
                    let name = &en.name;
                    exit_err!("DirEntry::fd(): openat() {name} {fd1} {fd2}");
                }
                if (fd1 != libc::AT_FDCWD && libc::close(fd1) < 0) {
                    exit_err!("DirEntry::fd(): close()");
                }

                fd1 = fd2;
            }
        }

        fd1

    }

    fn walk(&mut self) {
        let mut statbuf: libc::stat = unsafe { mem::zeroed() };
        let mut ent: *mut libc::dirent;
        let mut dir: *mut libc::DIR;
        let mut fd: i32;
        let mut d_name_var: &str = "";

        unsafe {
            let mut entry = self.parent.clone();
            while let Some(entry_ref) = entry {
                {
                    let e = entry_ref.borrow();
                    if self.dev == e.dev && self.ino == e.ino {
                        self.loop_found = true;
                        return;
                    }
                } 
                entry = entry_ref.borrow().parent.clone();
            }

            fd = self.fd();

            if self.parent.is_none() {
                    if (libc::fstat(fd, &mut statbuf as _) < 0) {
                        exit_err!("DirEntry::walk()3: fstat()");
                    }
                    self.dev = statbuf.st_dev;
                    self.ino = statbuf.st_ino;
                
            }

            dir = libc::fdopendir(fd);

            if (dir.is_null()) {
                exit_err!("DirEntry::walk(): fdopendir()");
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

                if d_name_var != "." && d_name_var != ".."{
                    let new_entry = DirEntry::new(d_name_var, self.options.clone(), Some(Rc::new(RefCell::new(self.clone()))));
                    self.entries.push_front(new_entry);
                }
            }
            if (libc::closedir(dir) < 0){
                exit_err!("walk(): closedir()");
            }
            fd = -1;

            for ref entry_ref in &self.entries {
                let mut entry = entry_ref.borrow_mut();
                if fd < 0 {
                    fd = self.fd();
                }
                let should_go_symlink = if self.options.follow_slink {
                    0
                } else {
                    libc::AT_SYMLINK_NOFOLLOW
                };
                
                if libc::fstatat(
                    fd,
                    str_to_cstring(&entry.name.to_string()).as_ptr(),
                    &mut statbuf as _,
                    should_go_symlink
                ) < 0
                {
                    if (*libc::__errno_location() == libc::ENOENT) {
                        continue;
                    }
                    if (*libc::__errno_location() == libc::ELOOP) {
                        continue;
                    }
                    exit_err!("DirEntry::walk()1: fstatat() 1");
                }
                let is_dir = s_isdir(statbuf.st_mode);

                if is_dir &&
                    self.options.follow_slink && libc::fstatat(
                        fd,
                        str_to_cstring(&entry.name.to_string()).as_ptr(),
                        &mut statbuf as _,
                        libc::AT_SYMLINK_NOFOLLOW
                    ) <0 {
                        exit_err!("DirEntry::walk()2: fstatat() 2");
                }

                entry.dev = statbuf.st_dev;
                entry.ino = statbuf.st_ino;

                if is_dir {
                    if (libc::close(fd) < 0) {
                        exit_err!("DirEntry::walk(): close()");
                    }
                    fd = -1;
                    entry.walk();
                }
            }

            if (fd >= 0 && libc::close(fd) < 0) {
                exit_err!("DirEntry::walk(): close()");
            }
        }
        
    }

    fn show(&self, level: usize) {
        let mut seen_inodes = HashSet::new();
        self.show_internal(level, &mut seen_inodes);
    }

    fn show_internal(&self, level: usize, seen_inodes: &mut HashSet<u64>) {
        if self.options.prevent_multiple_pathnames && seen_inodes.contains(&self.ino) {
            return; 
        }
        seen_inodes.insert(self.ino);

        print!("{:width$}", "", width = level);
        print!("{}", self.name);
        if self.loop_found {
            print!(" (loop)");
        }
        if self.cycle {
            print!(" (cycle)");
        }
        println!();

        for entry_ref in &self.entries {
            entry_ref.borrow().show_internal(level + 2, seen_inodes);
        }
    }

    fn show_default(&self) {
        self.show(0);
    }


}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (follow_slink, prohibit_mount_crossing, prohibit_non_subdir_transition, prevent_multiple_pathnames, directory_path) = parse_arguments::parse_arguments(&args);


    match directory_path {
        Some(d_path) => {
            unsafe {
                let opt = Options::new(follow_slink, prevent_multiple_pathnames);
                
                if(prohibit_non_subdir_transition) {
                    if libc::chroot(str_to_cstring("/").as_ptr()) < 0 {
                        exit_err!("chroot");
                    }
                    if libc::chdir(str_to_cstring(&d_path).as_ptr()) < 0 {
                        exit_err!("chdir");
                    }
                }
                
                let entry_ref = DirEntry::new(&d_path, opt, None);
                let mut entry = entry_ref.borrow_mut();
                entry.walk();
                entry.show_default();
            }
        }
        None => {
            exit_err!("workdir is empty");
        }
    }
}

