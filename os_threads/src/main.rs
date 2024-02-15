use std::time::Duration;

fn main() {
    println!("So, we start the program here!");

    let t1 = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(200));
        println!("The long running tasks finish at last!");
    });

    let t2 = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(100));
        println!("We can chain callbacks...");

        let t3 = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(50));
            println!("...like this!");
        });
        t3.join().unwrap();
    });

    println!("The tasks run concurrently!");
    t1.join().unwrap();
    t2.join().unwrap();
    // So, we start the program here!
    // The tasks run concurrently!
    // We can chain callbacks...
    // ...like this!
    // The long running tasks finish at last!
}
