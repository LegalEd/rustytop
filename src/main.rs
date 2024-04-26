// use std::fmt;
use std::path::Path;

use sysinfo::Uid;
use tabled::{Tabled, Table};
use tabled::settings::{style::{BorderColor}, Style, object::Rows, themes::Colorization, Color, Width, Panel};
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
println!("{0: <5} | {1: <10} | {2: <10} | {3: <10}", "PID", "Process", "Path", "Memory");

//container for processes
#[derive(Tabled)]
struct ProcessMap {
    pid: u32,
    name: &'static str,
    path: &'static str,
    user: &'static str,
    cpu: f32
}



let mut process_map: Vec<(u32, &str, Option<&Path>, Option<&Uid>, f32)> = Vec::new();

for (pid, process) in sys.processes() {
    process_map.push((pid.as_u32(), process.name(), process.exe(), process.user_id(), process.cpu_usage()));
}

process_map.sort_by_key(|k| k.0);



// create table



let mut table_process_map: Vec<(u32, &str, &str, String, f32)> = Vec::new();
let mut user_uid: String = String::new(); 

for (pid, process, path, owner, cpu_use) in process_map {
    match owner {
    Some(owner) => { println!("{}, {}, {:?}, {:?}, {:?}", pid, process, path.unwrap(), Some(get_user_by_uid(owner.to_string().parse::<u32>().unwrap()).unwrap().name()).unwrap(), cpu_use);
    user_uid = get_user_by_uid(owner.to_string().parse::<u32>().unwrap()).unwrap().name().to_os_string().as_os_str().to_str().unwrap().to_string() ;
    table_process_map.push((pid, process, path.unwrap().to_str().unwrap(), user_uid, cpu_use));
 },
    None => { println!("{}, {}, {:?}, None, {:?}", pid, process, path.unwrap(), cpu_use); },
}
}



// print table

let color_col1 = Color::BG_GREEN | Color::FG_BLACK;
let color_col2 = Color::BG_MAGENTA | Color::FG_BLACK;
let color_col3 = Color::BG_YELLOW | Color::FG_BLACK;
let color_head = Color::BG_WHITE | Color::FG_BLACK;
let color_head_text = Color::BG_BLUE | Color::FG_BLACK;

 let mut table = Table::new(table_process_map);
//  table.with(BorderColor::new().set_top(Color::FG_GREEN));
table
    .with(Panel::header("Running Processes"))
    .with(Style::empty())
    .with(Colorization::columns([color_col1, color_col2, color_col3]))
    .with(Colorization::exact([color_head], Rows::first()))
    .modify(Rows::first(), color_head_text);

table.modify(Rows::new(1..), Width::truncate(30).suffix("..."));
 println!("{table}");




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
