use libc::{c_int, c_void};
use std::{ffi::CString, ptr::null};

const TMPFS_PATH: &str = "/mnt/tmpfs-init";
const SIZE: &str = "4G";
const NO_DATA: [String; 0] = [];

fn main() -> ! {
    std::fs::create_dir_all(TMPFS_PATH).unwrap();
    mount("tmpfs", TMPFS_PATH, "tmpfs", 0, [format!("size={SIZE}")]);

    std::fs::create_dir_all(format!("{TMPFS_PATH}/work")).unwrap();
    std::fs::create_dir_all(format!("{TMPFS_PATH}/write")).unwrap();
    std::fs::create_dir_all(format!("{TMPFS_PATH}/root")).unwrap();

    mount(
        "overlay",
        &format!("{TMPFS_PATH}/root"),
        "overlay",
        0,
        [
            "lowerdir=/".to_string(),
            format!("upperdir={TMPFS_PATH}/write"),
            format!("workdir={TMPFS_PATH}/work"),
        ],
    );

    mount(
        "/dev",
        &format!("{TMPFS_PATH}/root/dev"),
        "bind",
        libc::MS_BIND,
        NO_DATA,
    );
    mount(
        "sysfs",
        &format!("{TMPFS_PATH}/root/sys"),
        "sysfs",
        0,
        NO_DATA,
    );
    mount(
        "proc",
        &format!("{TMPFS_PATH}/root/proc"),
        "proc",
        0,
        NO_DATA,
    );

    exec(
        [
            "/usr/sbin/chroot",
            format!("{TMPFS_PATH}/root/").as_str(),
            "/sbin/init",
        ]
    );
}

fn mount<I: IntoIterator>(src: &str, target: &str, fstype: &str, flags: u64, data: I)
where
    I::Item: std::fmt::Display,
{
    let src = CString::new(src).unwrap();
    let target = CString::new(target).unwrap();
    let fstype = CString::new(fstype).unwrap();
    let data = data
        .into_iter()
        .fold(None, |a, b| {
            Some(match a {
                Some(a) => format!("{a},{b}"),
                None => b.to_string(),
            })
        })
        .unwrap_or_default();
    let data = CString::new(data).unwrap();

    unsafe {
        libc::mount(
            src.as_ptr(),
            target.as_ptr(),
            fstype.as_ptr(),
            flags,
            data.as_ptr() as *const c_void,
        )
        .check_err(line!());
    }
}

fn exec<I: IntoIterator>(args: I) -> !
where
    I::Item: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|x| CString::new(x.as_ref()).unwrap())
        .collect::<Vec<_>>();
    let args = args
        .iter()
        .map(|x| x.as_ptr())
        .chain([null()])
        .collect::<Vec<_>>();

    let cmd = args.first().copied().unwrap();

    unsafe {
        libc::execvp(cmd, args.as_ptr()).check_err(line!());
    }

    unreachable!();
}

trait CheckErr: Sized {
    unsafe fn check_err(self, line: u32);
}
impl CheckErr for c_int {
    unsafe fn check_err(self, line: u32) {
        match self {
            0 => (),
            r => {
                let e = libc::__errno_location()
                    .as_mut()
                    .copied()
                    .unwrap_or_default();
                eprintln!("Line: {line}, Ret: {r}, Errorno: {e}");
                panic!("Line: {line}, Ret: {r}, Errorno: {e}");
            }
        }
    }
}
