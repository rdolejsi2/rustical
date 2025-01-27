use sys_info;

fn main() {
    let os_type: String = sys_info::os_type().unwrap_or("Unknown".to_string());
    let os_release: String = sys_info::os_release().unwrap_or("Unknown".to_string());
    println!("Running in OS {}, release {}", os_type, os_release);
}
