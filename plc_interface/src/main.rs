pub mod plc;

use plc::interface;

fn main() {
    println!("Hello, world!");

    
    let mut interface = interface::new("192.168.121.98:41100");

    println!("{:02x?}", interface.login(b"test22", b"test22"));


    let mut led = true;
    loop {
        interface.set_exten_bool(b"LED1", led);
        led = !led;

        std::thread::sleep(core::time::Duration::from_millis(1000));
    }
    
}

