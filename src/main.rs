use std::path::Path;

use sysinfo::Uid;
use users::{get_user_by_uid, get_current_uid};



fn main(){

use sysinfo::{
    Components, Disks, Networks, System
};

// "new_all" to ensure that all list of components, network interfaces,
// disks and users are already filled!
let mut sys = System::new_all();


let user = get_user_by_uid(get_current_uid()).unwrap();
println!("Hello, {}!", user.name().to_string_lossy());

// Update all information of our `System` struct.
sys.refresh_all();

// Display system information:
println!("=> system:");
println!("System name:             {:?}", System::name());
println!("System kernel version:   {:?}", System::kernel_version());
println!("System OS version:       {:?}", System::os_version());
println!("System host name:        {:?}", System::host_name());

// RAM and swap information:
println!("total memory: {} bytes", sys.total_memory());
println!("used memory : {} bytes", sys.used_memory());
println!("total swap  : {} bytes", sys.total_swap());
println!("used swap   : {} bytes", sys.used_swap());


// Number of CPUs:
println!("#CPUs: {}", sys.cpus().len());

// Display processes ID, name na disk usage:
println!("=> processes");

// container for processes

let mut process_map: Vec<(u32, &str, Option<&Path>, Option<&Uid>, f32)> = Vec::new();

for (pid, process) in sys.processes() {
    process_map.push((pid.as_u32(), process.name(), process.exe(), process.user_id(), process.cpu_usage()));
}

process_map.sort_by_key(|k| k.0);

for (pid, process, path, owner, cpu_use) in process_map {
    match owner {
    Some(owner) => { println!("{}, {}, {:?}, {:?}, {:?}", pid, process, path.unwrap(), Some(get_user_by_uid(owner.to_string().parse::<u32>().unwrap()).unwrap().name()).unwrap(), cpu_use); },
    None => { println!("{}, {}, {:?}, None, {:?}", pid, process, path.unwrap(), cpu_use); },
}
}

// for (pid, process) in sys.processes() {
//     process_map.push((pid.as_u32(), process.user_id()));
// }



// We display all disks' information:
println!("=> disks:");
let disks = Disks::new_with_refreshed_list();
for disk in &disks {
    println!("{disk:?}");
}

// Network interfaces name, total data received and total data transmitted:
let networks = Networks::new_with_refreshed_list();
println!("=> networks:");
for (interface_name, data) in &networks {
    println!(
        "{interface_name}: {} B (down) / {} B (up)",
        data.total_received(),
        data.total_transmitted(),
    );
    // If you want the amount of data received/transmitted since last call
    // to `Networks::refresh`, use `received`/`transmitted`.
}

// Components temperature:
let components = Components::new_with_refreshed_list();
println!("=> components:");
for component in &components {
    println!("{component:?}");
}
}
